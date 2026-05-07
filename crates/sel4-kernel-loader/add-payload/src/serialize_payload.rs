//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};
use std::ops::Range;
use std::path::Path;

use num::{CheckedAdd, CheckedSub, Integer, NumCast, One, PrimInt, traits::WrappingSub};
use object::ObjectSegment as _;
use object::{
    Object, ReadCache, ReadRef,
    elf::PT_LOAD,
    read::elf::{ElfFile, FileHeader, ProgramHeader},
};

use serde::{Deserialize, Serialize};

use sel4_kernel_loader_payload_types::*;

const PAGE_SIZE_BITS: usize = 12;

type Ranges = Vec<Range<u64>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlatformInfoForBuildSystem {
    memory: Ranges,
    devices: Ranges,
}

pub fn serialize_payload<T: FileHeader<Word: PrimInt + WrappingSub + Integer + Serialize>>(
    kernel_path: impl AsRef<Path>,
    app_path: impl AsRef<Path>,
    dtb_path: impl AsRef<Path>,
    platform_info_path: impl AsRef<Path>,
) -> Payload {
    let platform_info: PlatformInfoForBuildSystem =
        serde_yaml::from_reader(fs::File::open(&platform_info_path).unwrap()).unwrap();

    let mut builder = Builder::new();

    let kernel_image = with_elf(&kernel_path, |elf| {
        builder.add_image::<T, _>(elf, elf_phys_to_vaddr_offset(elf))
    });

    let user_image = with_elf(&app_path, |elf| {
        let virt_addr_range = elf_virt_addr_range(elf);
        let virt_footprint = coarsen_footprint(virt_addr_range, T::Word::one() << PAGE_SIZE_BITS);
        let footprint_size = virt_footprint
            .end
            .checked_sub(&virt_footprint.start)
            .unwrap();
        let phys_start = <T::Word as NumCast>::from(platform_info.memory.last().unwrap().end)
            .unwrap()
            .checked_sub(&footprint_size)
            .unwrap();
        let phys_to_virt_offset = phys_to_virt_offset_for(phys_start, virt_footprint.start);
        builder.add_image::<T, _>(elf, phys_to_virt_offset)
    });

    let fdt_content = fs::read(dtb_path).unwrap();
    let fdt_paddr = user_image.phys_addr_range.start.0
        - u64::try_from(fdt_content.len())
            .unwrap()
            .next_multiple_of(1 << PAGE_SIZE_BITS);
    let fdt_phys_addr_range = builder.add_region(fdt_paddr, fdt_content);

    Payload {
        info: PayloadInfo {
            kernel_image,
            user_image,
            fdt_phys_addr_range: Some(Word::from_u64_range(&fdt_phys_addr_range)),
        },
        data: builder.regions,
    }
}

//

struct Builder {
    regions: Vec<Region>,
}

impl Builder {
    fn new() -> Self {
        Self { regions: vec![] }
    }

    fn add_segments<'a, T: FileHeader<Word: PrimInt + WrappingSub>, R: ReadRef<'a>>(
        &mut self,
        elf: &ElfFile<'a, T, R>,
        phys_to_virt_offset: T::Word,
    ) {
        let endian = elf.endian();
        for seg in elf.segments() {
            let phdr = seg.elf_program_header();
            if phdr.p_type(endian) == PT_LOAD {
                let vaddr = phdr.p_vaddr(endian);
                let paddr = virt_to_phys(vaddr, phys_to_virt_offset);
                let filesz = phdr.p_filesz(endian);
                let memsz = phdr.p_memsz(endian);
                let data = seg.data().unwrap();
                self.add_region(paddr.into(), data.to_vec());
                if memsz > filesz {
                    self.regions.push(Region {
                        phys_addr_range: Word::from_u64_range(
                            &(paddr.checked_add(&filesz).unwrap()
                                ..paddr.checked_add(&memsz).unwrap()),
                        ),
                        content: None,
                    });
                }
            }
        }
    }

    fn add_region(&mut self, phys_addr_start: u64, content: Vec<u8>) -> Range<u64> {
        let phys_addr_range =
            phys_addr_start..(phys_addr_start + u64::try_from(content.len()).unwrap());
        self.regions.push(Region {
            phys_addr_range: Word::from_u64_range(&phys_addr_range),
            content: Some(content),
        });
        phys_addr_range
    }

    fn add_image<'a, T: FileHeader<Word: PrimInt + WrappingSub + Integer>, R: ReadRef<'a>>(
        &mut self,
        elf: &ElfFile<'a, T, R>,
        phys_to_virt_offset: T::Word,
    ) -> ImageInfo {
        let virt_addr_range = elf_virt_addr_range(elf);
        let phys_addr_range = {
            let start = virt_to_phys(virt_addr_range.start, phys_to_virt_offset);
            let end = virt_to_phys(virt_addr_range.end, phys_to_virt_offset);
            Word::from_u64_range(&coarsen_footprint(
                start..end,
                T::Word::one() << PAGE_SIZE_BITS,
            ))
        };
        let virt_entry = elf.entry().into();
        self.add_segments(elf, phys_to_virt_offset);
        ImageInfo {
            phys_addr_range,
            phys_to_virt_offset: phys_to_virt_offset.into().into(),
            virt_entry,
        }
    }
}

//

fn with_elf<T: FileHeader, R, F>(path: impl AsRef<Path>, f: F) -> R
where
    F: FnOnce(&ElfFile<T, &ReadCache<File>>) -> R,
{
    let file = File::open(path).unwrap();
    let read_cache = ReadCache::new(file);
    let elf = ElfFile::<T, _>::parse(&read_cache).unwrap();
    f(&elf)
}

fn elf_virt_addr_range<'a, T: FileHeader<Word: PrimInt>, R: ReadRef<'a>>(
    elf: &ElfFile<'a, T, R>,
) -> Range<T::Word> {
    let endian = elf.endian();
    let virt_min = elf
        .elf_program_headers()
        .iter()
        .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
        .map(|phdr| phdr.p_vaddr(endian))
        .min()
        .unwrap();
    let virt_max = elf
        .elf_program_headers()
        .iter()
        .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
        .map(|phdr| {
            phdr.p_vaddr(endian)
                .checked_add(&phdr.p_memsz(endian))
                .unwrap()
        })
        .max()
        .unwrap();
    virt_min..virt_max
}

fn elf_phys_to_vaddr_offset<'a, T: FileHeader<Word: PrimInt + WrappingSub>, R: ReadRef<'a>>(
    elf: &ElfFile<'a, T, R>,
) -> T::Word {
    let endian = elf.endian();
    unified(
        elf.elf_program_headers()
            .iter()
            .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
            .map(|phdr| phys_to_virt_offset_for(phdr.p_paddr(endian), phdr.p_vaddr(endian))),
    )
}

//

fn coarsen_footprint<T: PrimInt + Integer>(footprint: Range<T>, granularity: T) -> Range<T> {
    let start = footprint.start.prev_multiple_of(&granularity);
    let end = footprint.end.next_multiple_of(&granularity);
    start..end
}

fn virt_to_phys<T: PrimInt + WrappingSub>(vaddr: T, phys_to_virt_offset: T) -> T {
    vaddr.wrapping_sub(&phys_to_virt_offset)
}

fn phys_to_virt_offset_for<T: PrimInt + WrappingSub>(paddr: T, vaddr: T) -> T {
    vaddr.wrapping_sub(&paddr)
}

fn unified<T: Eq>(mut it: impl Iterator<Item = T>) -> T {
    let first = it.next().unwrap();
    assert!(it.all(|subsequent| subsequent == first));
    first
}
