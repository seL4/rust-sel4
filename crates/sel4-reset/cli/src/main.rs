//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;
use std::path::PathBuf;
use std::{any, fs};

use clap::Parser;

use anyhow::Error;
use num::{NumCast, ToPrimitive};
use object::elf::{FileHeader32, FileHeader64, PF_R, PT_PHDR, ProgramHeader32, ProgramHeader64};
use object::elf::{PF_W, PT_LOAD};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endian, File, Object, ObjectSection, ObjectSegment, ObjectSymbol, U32, U64, pod};
use rangemap::RangeSet;

// HACK
const PAGE_SIZE: u64 = 4096;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    in_file_path: PathBuf,
    #[arg(short = 'o')]
    out_file_path: PathBuf,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let in_perms = fs::metadata(&cli.in_file_path)?.permissions();
    let in_bytes = fs::read(&cli.in_file_path)?;
    let in_file = File::parse(in_bytes.as_slice())?;

    let out_bytes = match in_file {
        File::Elf32(elf) => continue_with_type(&elf),
        File::Elf64(elf) => continue_with_type(&elf),
        _ => {
            panic!()
        }
    }?;

    fs::write(&cli.out_file_path, &out_bytes)?;
    fs::set_permissions(&cli.out_file_path, in_perms)?;

    Ok(())
}

fn continue_with_type<'a, T>(orig_elf: &'a ElfFile<'a, T>) -> Result<Vec<u8>, Error>
where
    T: FileHeader<Word: NumCast + PatchValue> + PatchPhoff,
{
    let mut x = X::new(orig_elf);
    x.add_regions();
    let data = x.finalize();
    Ok(data)
}

struct X<'a, T: FileHeader> {
    orig_elf: &'a ElfFile<'a, T>,
    phdrs: Vec<T::ProgramHeader>,
    data: Vec<u8>,
}

pub trait PatchValue {
    fn to_bytes(&self, endian: impl Endian) -> Vec<u8>;
}

impl PatchValue for u32 {
    fn to_bytes(&self, endian: impl Endian) -> Vec<u8> {
        endian.write_u32_bytes(*self).to_vec()
    }
}

impl PatchValue for u64 {
    fn to_bytes(&self, endian: impl Endian) -> Vec<u8> {
        endian.write_u64_bytes(*self).to_vec()
    }
}

struct RegionMeta<T: FileHeader> {
    dst_vaddr: T::Word,
    dst_size: T::Word,
    src_vaddr: T::Word,
    src_size: T::Word,
}

impl<T: FileHeader<Word: PatchValue>> RegionMeta<T> {
    fn pack(&self, endian: impl Endian, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.dst_vaddr.to_bytes(endian));
        buf.extend_from_slice(&self.dst_size.to_bytes(endian));
        buf.extend_from_slice(&self.src_vaddr.to_bytes(endian));
        buf.extend_from_slice(&self.src_size.to_bytes(endian));
    }
}

impl<'a, T: FileHeader<Word: NumCast + PatchValue> + PatchPhoff> X<'a, T> {
    fn new(orig_elf: &'a ElfFile<'a, T>) -> Self {
        Self {
            orig_elf,
            phdrs: orig_elf.elf_program_headers().to_vec(),
            data: orig_elf.data().to_vec(),
        }
    }

    fn endian(&self) -> T::Endian {
        self.orig_elf.endian()
    }

    fn footprint(&self) -> Option<Range<u64>> {
        let start = self
            .phdrs
            .iter()
            .map(|phdr| phdr.p_vaddr(self.endian()).into())
            .min()?;
        let end = self
            .phdrs
            .iter()
            .map(|phdr| phdr.p_vaddr(self.endian()).into() + phdr.p_memsz(self.endian()).into())
            .max()?;
        Some(start..end)
    }

    fn next_aligned_vaddr(&self, align: u64) -> u64 {
        self.footprint()
            .map(|footprint| footprint.end)
            .unwrap_or(0)
            .next_multiple_of(align.max(1))
    }

    fn align_data_cursor(&mut self, align: u64) {
        self.data.resize(
            self.data.len().next_multiple_of(align.try_into().unwrap()),
            0,
        );
    }

    #[allow(dead_code)]
    fn segment_data(&mut self, phdr: &T::ProgramHeader) -> &mut [u8] {
        let endian = self.endian();
        let offset = phdr.p_offset(endian).to_usize().unwrap();
        let filesz = phdr.p_filesz(endian).to_usize().unwrap();
        &mut self.data[offset..][..filesz]
    }

    fn add_segment(
        &mut self,
        p_type: u32,
        p_flags: u32,
        p_memsz: u64,
        p_align: u64,
        data_align: u64,
        data: &[u8],
    ) -> T::ProgramHeader {
        assert!(data_align <= p_align);
        self.align_data_cursor(data_align);
        let p_offset = self.data.len().try_into().unwrap();
        let p_filesz = data.len().try_into().unwrap();
        self.data.extend_from_slice(data);
        self.add_segment_raw(GenericProgramHeader {
            p_type,
            p_flags,
            p_offset,
            p_vaddr: 0,
            p_paddr: 0,
            p_filesz,
            p_memsz,
            p_align,
        })
    }

    fn add_segment_raw(&mut self, mut phdr: GenericProgramHeader) -> T::ProgramHeader {
        let p_vaddr = self.next_aligned_vaddr(phdr.p_align) + phdr.p_offset % phdr.p_align.max(1);
        phdr.p_vaddr = p_vaddr;
        phdr.p_paddr = p_vaddr;
        let phdr = T::convert_phdr(self.endian(), &phdr);
        self.phdrs.push(phdr);
        phdr
    }

    fn patch_word(&mut self, symbol_name: &str, value: T::Word) {
        let value_bytes = value.to_bytes(self.endian());
        let symbol = self.orig_elf.symbol_by_name(symbol_name).unwrap();
        let symbol_vaddr = symbol.address();
        assert_eq!(usize::try_from(symbol.size()).unwrap(), value_bytes.len());
        let offset_in_file = self
            .orig_elf
            .segments()
            .find_map(|segment| {
                let seg_mem_start = segment.address();
                let seg_mem_end = seg_mem_start + segment.size();
                if (seg_mem_start..seg_mem_end).contains(&symbol_vaddr) {
                    let offset_in_seg = symbol_vaddr - seg_mem_start;
                    let (seg_file_start, seg_file_size) = segment.file_range();
                    assert!(
                        offset_in_seg + u64::try_from(value_bytes.len()).unwrap() <= seg_file_size
                    );
                    Some(seg_file_start + offset_in_seg)
                } else {
                    None
                }
            })
            .unwrap();
        self.data[usize::try_from(offset_in_file).unwrap()..][..value_bytes.len()]
            .copy_from_slice(&value_bytes);
    }

    pub fn patch_word_with_cast(&mut self, symbol_name: &str, value: impl ToPrimitive + Clone)
    where
        T::Word: PatchValue + NumCast,
    {
        self.patch_word(
            symbol_name,
            <T::Word as NumCast>::from(value.clone()).unwrap_or_else(|| {
                panic!(
                    "value {:#x?} out of bounds for word type {}",
                    value.to_u64().unwrap(),
                    any::type_name::<T::Word>()
                )
            }),
        )
    }

    fn add_all_phdrs(&mut self) -> T::ProgramHeader {
        let endian = self.endian();
        let phdrs_load_phdr = {
            let data_align = align_of::<T::ProgramHeader>().try_into().unwrap();
            let eventual_n = self.phdrs.len() + 1;
            let data_size = size_of::<T>() + eventual_n * size_of::<T::ProgramHeader>();
            self.add_segment(
                PT_LOAD,
                PF_R,
                data_size.try_into().unwrap(),
                PAGE_SIZE,
                data_align,
                &vec![0; data_size],
            )
        };
        let mut phdrs_phdr_phdr = phdrs_load_phdr;
        T::set_p_type(&mut phdrs_phdr_phdr, endian, PT_PHDR);
        T::take_offset(
            &mut phdrs_phdr_phdr,
            endian,
            <T::Word as NumCast>::from(size_of::<T>()).unwrap(),
        );
        for phdr in self.phdrs.iter_mut() {
            if phdr.p_type(endian) == PT_PHDR {
                *phdr = phdrs_phdr_phdr;
            }
        }
        {
            let offset = phdrs_load_phdr.p_offset(endian).to_usize().unwrap();
            let filesz = phdrs_load_phdr.p_filesz(endian).to_usize().unwrap();
            let seg_data = &mut self.data[offset..][..filesz];
            let (ehdr_data, phdrs_data) = seg_data.split_at_mut(size_of::<T>());
            let mut new_ehdr = *self.orig_elf.elf_header();
            new_ehdr.patch_fake_header(self.phdrs.len());
            ehdr_data.copy_from_slice(pod::bytes_of(&new_ehdr));
            phdrs_data.copy_from_slice(pod::bytes_of_slice(&self.phdrs));
        }
        self.patch_word_with_cast("HACK__ehdr_start", phdrs_load_phdr.p_vaddr(endian));
        phdrs_phdr_phdr
    }

    fn add_regions(&mut self) {
        let endian = self.endian();

        let persistent_ranges = {
            let mut set = RangeSet::new();
            for s in self.orig_elf.sections() {
                if let Ok(name) = s.name()
                    && (name == ".persistent" || name.starts_with(".persistent."))
                {
                    set.insert(s.address()..(s.address() + s.size()));
                }
            }
            set
        };

        let mut regions: Vec<RegionMeta<T>> = vec![];
        for seg in self.orig_elf.segments() {
            let phdr = seg.elf_program_header();
            if phdr.p_type(endian) == PT_LOAD && phdr.p_flags(endian) & PF_W != 0 {
                let p_align = phdr.p_align(endian).into();
                let p_filesz = phdr.p_filesz(endian).into();
                let alt_phdr = self.add_segment_raw(GenericProgramHeader {
                    p_type: PT_LOAD,
                    p_flags: PF_R,
                    p_offset: phdr.p_offset(endian).into(),
                    p_vaddr: 0,
                    p_paddr: 0,
                    p_filesz,
                    p_memsz: p_filesz,
                    p_align,
                });
                {
                    let vaddr = phdr.p_vaddr(endian).into();
                    let memsz = phdr.p_memsz(endian).into();
                    let segment_range = vaddr..(vaddr + memsz);
                    let relevant_persistent_ranges = RangeSet::from_iter(
                        persistent_ranges
                            .intersection(&RangeSet::from_iter([segment_range.clone()])),
                    );
                    for ephermal in relevant_persistent_ranges.gaps(&segment_range) {
                        let region_memsz = ephermal.end - ephermal.start;
                        let region_offset_in_segment = ephermal.start - vaddr;
                        let (src_vaddr, src_size) = if region_offset_in_segment
                            < alt_phdr.p_filesz(endian).into()
                        {
                            let start = alt_phdr.p_vaddr(endian).into() + region_offset_in_segment;
                            (
                                start,
                                alt_phdr.p_filesz(endian).into().min(ephermal.end - vaddr),
                            )
                        } else {
                            (0, 0)
                        };
                        regions.push(RegionMeta {
                            dst_vaddr: <T::Word as NumCast>::from(ephermal.start).unwrap(),
                            dst_size: <T::Word as NumCast>::from(region_memsz).unwrap(),
                            src_vaddr: <T::Word as NumCast>::from(src_vaddr).unwrap(),
                            src_size: <T::Word as NumCast>::from(src_size).unwrap(),
                        });
                    }
                }
            }
        }
        let regions_meta_data = {
            let mut v = vec![];
            for m in regions.iter() {
                m.pack(endian, &mut v);
            }
            v
        };
        let regions_meta_phdr = self.add_segment(
            PT_LOAD,
            PF_R,
            regions_meta_data.len().try_into().unwrap(),
            PAGE_SIZE,
            align_of::<RegionMeta<T>>().try_into().unwrap(),
            &regions_meta_data,
        );
        self.patch_word_with_cast(
            "sel4_reset_regions_meta_vaddr",
            regions_meta_phdr.p_vaddr(endian),
        );
        self.patch_word_with_cast("sel4_reset_regions_meta_count", regions.len());
    }

    fn finalize(mut self) -> Vec<u8> {
        let endian = self.endian();
        let all_phdrs_phdr = self.add_all_phdrs();
        let (ehdr, _) = pod::from_bytes_mut::<T>(&mut self.data).unwrap();
        ehdr.patch_header(all_phdrs_phdr.p_offset(endian), self.phdrs.len());
        self.data
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GenericProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

trait PatchPhoff: FileHeader {
    fn patch_header(&mut self, e_phoff: Self::Word, e_phnum: usize);
    fn patch_fake_header(&mut self, e_phnum: usize);
    fn convert_phdr(endian: Self::Endian, generic: &GenericProgramHeader) -> Self::ProgramHeader;
    fn set_p_type(phdr: &mut Self::ProgramHeader, endian: Self::Endian, p_type: u32);
    fn take_offset(phdr: &mut Self::ProgramHeader, endian: Self::Endian, n: Self::Word);
}

impl<E: Endian> PatchPhoff for FileHeader32<E> {
    fn patch_header(&mut self, e_phoff: Self::Word, e_phnum: usize) {
        self.e_phoff.set(self.endian().unwrap(), e_phoff);
        self.e_phnum
            .set(self.endian().unwrap(), e_phnum.try_into().unwrap());
        self.e_phnum.set(
            self.endian().unwrap(),
            (e_phnum * size_of::<Self::ProgramHeader>())
                .try_into()
                .unwrap(),
        );
    }

    fn patch_fake_header(&mut self, e_phnum: usize) {
        self.e_phnum
            .set(self.endian().unwrap(), e_phnum.try_into().unwrap());
        self.e_phnum.set(
            self.endian().unwrap(),
            (e_phnum * size_of::<Self::ProgramHeader>())
                .try_into()
                .unwrap(),
        );
    }

    fn convert_phdr(endian: Self::Endian, generic: &GenericProgramHeader) -> Self::ProgramHeader {
        ProgramHeader32 {
            p_type: U32::new(endian, generic.p_type),
            p_offset: U32::new(endian, generic.p_offset.try_into().unwrap()),
            p_vaddr: U32::new(endian, generic.p_vaddr.try_into().unwrap()),
            p_paddr: U32::new(endian, generic.p_paddr.try_into().unwrap()),
            p_filesz: U32::new(endian, generic.p_filesz.try_into().unwrap()),
            p_memsz: U32::new(endian, generic.p_memsz.try_into().unwrap()),
            p_flags: U32::new(endian, generic.p_flags),
            p_align: U32::new(endian, generic.p_align.try_into().unwrap()),
        }
    }

    fn set_p_type(phdr: &mut Self::ProgramHeader, endian: Self::Endian, p_type: u32) {
        phdr.p_type.set(endian, p_type)
    }

    fn take_offset(phdr: &mut Self::ProgramHeader, endian: Self::Endian, n: Self::Word) {
        phdr.p_offset.set(endian, phdr.p_offset.get(endian) + n);
        phdr.p_vaddr.set(endian, phdr.p_vaddr.get(endian) + n);
        phdr.p_paddr.set(endian, phdr.p_paddr.get(endian) + n);
        phdr.p_filesz
            .set(endian, phdr.p_filesz.get(endian).saturating_sub(n));
        phdr.p_memsz.set(endian, phdr.p_memsz.get(endian) - n);
    }
}

impl<E: Endian> PatchPhoff for FileHeader64<E> {
    fn patch_header(&mut self, e_phoff: Self::Word, e_phnum: usize) {
        self.e_phoff.set(self.endian().unwrap(), e_phoff);
        self.e_phnum
            .set(self.endian().unwrap(), e_phnum.try_into().unwrap());
    }

    fn patch_fake_header(&mut self, e_phnum: usize) {
        self.e_phnum
            .set(self.endian().unwrap(), e_phnum.try_into().unwrap());
        self.e_phnum.set(
            self.endian().unwrap(),
            (e_phnum * size_of::<Self::ProgramHeader>())
                .try_into()
                .unwrap(),
        );
    }

    fn convert_phdr(endian: Self::Endian, generic: &GenericProgramHeader) -> Self::ProgramHeader {
        ProgramHeader64 {
            p_type: U32::new(endian, generic.p_type),
            p_flags: U32::new(endian, generic.p_flags),
            p_offset: U64::new(endian, generic.p_offset),
            p_vaddr: U64::new(endian, generic.p_vaddr),
            p_paddr: U64::new(endian, generic.p_paddr),
            p_filesz: U64::new(endian, generic.p_filesz),
            p_memsz: U64::new(endian, generic.p_memsz),
            p_align: U64::new(endian, generic.p_align),
        }
    }

    fn set_p_type(phdr: &mut Self::ProgramHeader, endian: Self::Endian, p_type: u32) {
        phdr.p_type.set(endian, p_type)
    }

    fn take_offset(phdr: &mut Self::ProgramHeader, endian: Self::Endian, n: Self::Word) {
        phdr.p_offset.set(endian, phdr.p_offset.get(endian) + n);
        phdr.p_vaddr.set(endian, phdr.p_vaddr.get(endian) + n);
        phdr.p_paddr.set(endian, phdr.p_paddr.get(endian) + n);
        phdr.p_filesz
            .set(endian, phdr.p_filesz.get(endian).saturating_sub(n));
        phdr.p_memsz.set(endian, phdr.p_memsz.get(endian) - n);
    }
}
