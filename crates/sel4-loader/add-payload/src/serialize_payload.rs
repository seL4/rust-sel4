use std::fs::{self, File};
use std::ops::Range;
use std::path::Path;

use heapless::Vec as HeaplessVec;
use object::{
    elf::PT_LOAD,
    endian::Endianness,
    read::elf::{ElfFile, ProgramHeader},
    Object, ReadCache, ReadRef,
};

use serde::{Deserialize, Serialize};

use sel4_loader_payload_types::*;

const PAGE_SIZE: u64 = 4096;

// sel4_config::sel4_cfg_if! {
//     if #[cfg(WORD_SIZE = "64")] {
//         type FileHeader<T> = object::elf::FileHeader64<T>;
//     } else if #[cfg(WORD_SIZE = "32")] {
//         type FileHeader<T> = object::elf::FileHeader32<T>;
//     }
// }

// TODO
type FileHeader<T> = object::elf::FileHeader64<T>;

type Ranges = Vec<Range<u64>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlatformInfoForBuildSystem {
    memory: Ranges,
    devices: Ranges,
}

pub fn serialize_payload<'a>(
    kernel_path: impl AsRef<Path>,
    app_path: impl AsRef<Path>,
    dtb_path: impl AsRef<Path>,
    platform_info_path: impl AsRef<Path>,
) -> Vec<u8> {
    let platform_info: PlatformInfoForBuildSystem =
        serde_yaml::from_reader(fs::File::open(&platform_info_path).unwrap()).unwrap();

    let mut builder = Builder::new();

    let kernel_image = with_elf(&kernel_path, |elf| {
        builder.add_image(elf, elf_phys_to_vaddr_offset(elf))
    });

    let user_image = with_elf(&app_path, |elf| {
        let virt_addr_range = elf_virt_addr_range(elf);
        let virt_footprint = coarsen_footprint(virt_addr_range, PAGE_SIZE);
        let virt_footprint_size = virt_footprint.end - virt_footprint.start;
        let phys_start = platform_info.memory.last().unwrap().end - virt_footprint_size;
        let phys_to_virt_offset = phys_to_virt_offset_for(phys_start, virt_footprint.start);
        builder.add_image(elf, phys_to_virt_offset)
    });

    let fdt_content = fs::read(dtb_path).unwrap();
    let fdt_paddr = user_image.phys_addr_range.start
        - u64::try_from(fdt_content.len())
            .unwrap()
            .next_multiple_of(PAGE_SIZE);
    let fdt_phys_addr_range = builder.add_region(fdt_paddr, fdt_content);

    let payload = PayloadForX {
        info: PayloadInfo {
            kernel_image,
            user_image,
            fdt_phys_addr_range: Some(fdt_phys_addr_range),
        },
        data: builder.regions,
    };

    let mut blob = postcard::to_allocvec(&payload).unwrap();
    blob.extend(&builder.actual_content);
    blob
}

//

struct Builder {
    regions: HeaplessVec<Region<IndirectRegionContent>, MAX_NUM_REGIONS>,
    actual_content: Vec<u8>,
}

impl Builder {
    fn new() -> Self {
        Self {
            regions: HeaplessVec::new(),
            actual_content: vec![],
        }
    }

    fn add_segments<'a, T: ReadRef<'a>>(
        &mut self,
        elf: &ElfFile<'a, FileHeader<Endianness>, T>,
        phys_to_virt_offset: i64,
    ) {
        let endian = elf.endian();
        for phdr in elf
            .raw_segments()
            .iter()
            .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
        {
            let offset = phdr.p_offset(endian);
            let vaddr = phdr.p_vaddr(endian);
            let paddr = virt_to_phys(vaddr, phys_to_virt_offset);
            let filesz = phdr.p_filesz(endian);
            let memsz = phdr.p_memsz(endian);
            let content = elf
                .data()
                .read_bytes_at(offset.into(), filesz.into())
                .unwrap();
            self.add_region(paddr, content.to_vec());
            if memsz > filesz {
                self.regions
                    .push(Region {
                        phys_addr_range: (paddr + filesz)..(paddr + memsz),
                        content: None,
                    })
                    .unwrap();
            }
        }
    }

    fn add_region(&mut self, phys_addr_start: u64, content: Vec<u8>) -> Range<u64> {
        let phys_addr_range =
            phys_addr_start..(phys_addr_start + u64::try_from(content.len()).unwrap());
        self.regions
            .push(Region {
                phys_addr_range: phys_addr_range.clone(),
                content: Some(IndirectRegionContent {
                    content_range: {
                        let start = self.actual_content.len();
                        start..start + content.len()
                    },
                }),
            })
            .unwrap();
        self.actual_content.extend(content);
        phys_addr_range
    }

    fn add_image<'a, T: ReadRef<'a>>(
        &mut self,
        elf: &ElfFile<'a, FileHeader<Endianness>, T>,
        phys_to_virt_offset: i64,
    ) -> ImageInfo {
        let virt_addr_range = elf_virt_addr_range(elf);
        let phys_start = virt_to_phys(virt_addr_range.start, phys_to_virt_offset);
        let phys_end = virt_to_phys(virt_addr_range.end, phys_to_virt_offset);
        let phys_addr_range = coarsen_footprint(phys_start..phys_end, PAGE_SIZE);
        let virt_entry = elf.entry().try_into().unwrap();
        self.add_segments(elf, phys_to_virt_offset);
        ImageInfo {
            phys_addr_range,
            phys_to_virt_offset,
            virt_entry,
        }
    }
}

//

fn with_elf<T, F>(path: impl AsRef<Path>, f: F) -> T
where
    F: FnOnce(&ElfFile<FileHeader<Endianness>, &ReadCache<File>>) -> T,
{
    let file = File::open(path).unwrap();
    let read_cache = ReadCache::new(file);
    let elf = ElfFile::<FileHeader<Endianness>, _>::parse(&read_cache).unwrap();
    f(&elf)
}

fn elf_virt_addr_range<'a, T: ReadRef<'a>>(
    elf: &ElfFile<'a, FileHeader<Endianness>, T>,
) -> Range<u64> {
    let endian = elf.endian();
    let virt_min = elf
        .raw_segments()
        .iter()
        .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
        .map(|phdr| phdr.p_vaddr(endian))
        .min()
        .unwrap()
        .into();
    let virt_max = elf
        .raw_segments()
        .iter()
        .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
        .map(|phdr| phdr.p_vaddr(endian) + phdr.p_memsz(endian))
        .max()
        .unwrap()
        .into();
    virt_min..virt_max
}

fn elf_phys_to_vaddr_offset<'a, T: ReadRef<'a>>(
    elf: &ElfFile<'a, FileHeader<Endianness>, T>,
) -> i64 {
    let endian = elf.endian();
    unified(
        elf.raw_segments()
            .iter()
            .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
            .map(|phdr| phys_to_virt_offset_for(phdr.p_paddr(endian), phdr.p_vaddr(endian))),
    )
}

//

fn coarsen_footprint(footprint: Range<u64>, granularity: u64) -> Range<u64> {
    (footprint.start & !(granularity - 1))..footprint.end.next_multiple_of(granularity)
}

fn virt_to_phys(vaddr: u64, phys_to_virt_offset: i64) -> u64 {
    u64::try_from(i64::try_from(vaddr).unwrap() - phys_to_virt_offset).unwrap()
}

fn phys_to_virt_offset_for(paddr: u64, vaddr: u64) -> i64 {
    i64::try_from(vaddr).unwrap() - i64::try_from(paddr).unwrap()
}

fn unified<T: Eq>(mut it: impl Iterator<Item = T>) -> T {
    let first = it.next().unwrap();
    assert!(it.all(|subsequent| &subsequent == &first));
    first
}
