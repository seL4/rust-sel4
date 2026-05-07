//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;

use object::elf::{
    FileHeader32, FileHeader64, PF_R, PT_LOAD, PT_PHDR, ProgramHeader32, ProgramHeader64,
};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endian, Object as _, ObjectSegment as _, ObjectSymbol as _, Pod, U32, U64, pod};

pub struct Patching<'a, T: FileHeader> {
    orig_elf: &'a ElfFile<'a, T>,
    phdrs: Vec<T::ProgramHeader>,
    data: Vec<u8>,
}

impl<'a, T: FileHeaderExt> Patching<'a, T> {
    pub fn new(orig_elf: &'a ElfFile<'a, T>) -> Self {
        Self {
            orig_elf,
            phdrs: orig_elf.elf_program_headers().to_vec(),
            data: orig_elf.data().to_vec(),
        }
    }

    pub fn orig_elf(&self) -> &'a ElfFile<'a, T> {
        self.orig_elf
    }

    pub fn endian(&self) -> T::Endian {
        self.orig_elf().endian()
    }

    pub fn add_phdr(&mut self, phdr: T::ProgramHeader) -> &T::ProgramHeader {
        self.phdrs.push_mut(phdr)
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

    pub fn next_aligned_vaddr(&self, align: u64) -> u64 {
        self.footprint()
            .map(|footprint| footprint.end)
            .unwrap_or(0)
            .next_multiple_of(align.max(1))
    }

    pub fn align_data_cursor(&mut self, align: u64) {
        self.data.resize(
            self.data.len().next_multiple_of(align.try_into().unwrap()),
            0,
        );
    }

    pub fn add_segment(
        &mut self,
        mut phdr: GenericProgramHeader,
        data_align: u64,
        data: &[u8],
    ) -> &T::ProgramHeader {
        assert!(data_align <= phdr.p_align);
        self.align_data_cursor(data_align);
        phdr.p_offset = self.data.len().try_into().unwrap();
        phdr.p_filesz = data.len().try_into().unwrap();
        self.data.extend_from_slice(data);
        self.add_segment_raw(phdr)
    }

    pub fn add_segment_raw(&mut self, mut phdr: GenericProgramHeader) -> &T::ProgramHeader {
        let p_vaddr = self.next_aligned_vaddr(phdr.p_align) + phdr.p_offset % phdr.p_align.max(1);
        phdr.p_vaddr = p_vaddr;
        phdr.p_paddr = p_vaddr;
        self.add_phdr(T::ProgramHeader::from_generic(self.endian(), &phdr))
    }

    fn patch_symbol(&mut self, symbol_name: &str, value: &[u8]) {
        let symbol = self.orig_elf.symbol_by_name(symbol_name).unwrap();
        let symbol_vaddr = symbol.address();
        assert_eq!(usize::try_from(symbol.size()).unwrap(), value.len());
        let offset_in_file = self
            .orig_elf
            .segments()
            .find_map(|segment| {
                let seg_mem_start = segment.address();
                let seg_mem_end = seg_mem_start + segment.size();
                if (seg_mem_start..seg_mem_end).contains(&symbol_vaddr) {
                    let offset_in_seg = symbol_vaddr - seg_mem_start;
                    let (seg_file_start, seg_file_size) = segment.file_range();
                    assert!(offset_in_seg + u64::try_from(value.len()).unwrap() <= seg_file_size);
                    Some(seg_file_start + offset_in_seg)
                } else {
                    None
                }
            })
            .unwrap();
        self.data[usize::try_from(offset_in_file).unwrap()..][..value.len()].copy_from_slice(value);
    }

    pub fn finalize(mut self, phdrs_load_segment_p_align: u64) -> Vec<u8> {
        let endian = self.endian();
        let phdrs_load_phdr = {
            let data_align = align_of::<T::ProgramHeader>().try_into().unwrap();
            let eventual_n = self.phdrs.len() + 1;
            let data_size = eventual_n * size_of::<T::ProgramHeader>();
            let p_align = phdrs_load_segment_p_align;
            assert!(data_align <= p_align);
            self.align_data_cursor(data_align);
            let p_offset = self.data.len().try_into().unwrap();
            let p_filesz = data_size.try_into().unwrap();
            let p_vaddr = self.next_aligned_vaddr(p_align) + p_offset % p_align.max(1);
            T::ProgramHeader::from_generic(
                endian,
                &GenericProgramHeader {
                    p_type: PT_LOAD,
                    p_flags: PF_R,
                    p_offset,
                    p_vaddr,
                    p_paddr: p_vaddr,
                    p_filesz,
                    p_memsz: p_filesz,
                    p_align,
                },
            )
        };
        self.phdrs.push(phdrs_load_phdr);
        {
            let mut phdrs_phdr_phdr = phdrs_load_phdr;
            phdrs_phdr_phdr.set_p_type(endian, PT_PHDR);
            for phdr in self.phdrs.iter_mut() {
                if phdr.p_type(endian) == PT_PHDR {
                    *phdr = phdrs_phdr_phdr;
                }
            }
        }
        self.data
            .extend_from_slice(pod::bytes_of_slice(&self.phdrs));
        let (ehdr, _) = pod::from_bytes_mut::<T>(&mut self.data).unwrap();
        ehdr.set_e_phoff(endian, phdrs_load_phdr.p_offset(endian));
        ehdr.set_e_phnum(endian, self.phdrs.len().try_into().unwrap());
        self.patch_symbol(
            "sel4_phdrs_patched__vaddr",
            &phdrs_load_phdr.p_vaddr(endian).write_bytes(endian),
        );
        self.patch_symbol(
            "sel4_phdrs_patched__phnum",
            &endian.write_u16_bytes(u16::try_from(self.phdrs.len()).unwrap()),
        );
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

pub trait FileHeaderExt: FileHeader<Word: WordExt, ProgramHeader: ProgramHeaderExt> {
    fn set_e_phoff(&mut self, endian: Self::Endian, e_phoff: Self::Word);
    fn set_e_phnum(&mut self, endian: Self::Endian, e_phnum: u16);
}

impl<E: Endian> FileHeaderExt for FileHeader32<E> {
    fn set_e_phoff(&mut self, endian: Self::Endian, e_phoff: Self::Word) {
        self.e_phoff.set(endian, e_phoff)
    }

    fn set_e_phnum(&mut self, endian: Self::Endian, e_phnum: u16) {
        self.e_phnum.set(endian, e_phnum)
    }
}

impl<E: Endian> FileHeaderExt for FileHeader64<E> {
    fn set_e_phoff(&mut self, endian: Self::Endian, e_phoff: Self::Word) {
        self.e_phoff.set(endian, e_phoff)
    }

    fn set_e_phnum(&mut self, endian: Self::Endian, e_phnum: u16) {
        self.e_phnum.set(endian, e_phnum)
    }
}

pub trait WordExt: Pod {
    fn write_bytes(&self, endian: impl Endian) -> Vec<u8>;
}

impl WordExt for u32 {
    fn write_bytes(&self, endian: impl Endian) -> Vec<u8> {
        endian.write_u32_bytes(*self).to_vec()
    }
}

impl WordExt for u64 {
    fn write_bytes(&self, endian: impl Endian) -> Vec<u8> {
        endian.write_u64_bytes(*self).to_vec()
    }
}

pub trait ProgramHeaderExt: ProgramHeader {
    fn from_generic(endian: Self::Endian, generic: &GenericProgramHeader) -> Self;
    fn set_p_type(&mut self, endian: Self::Endian, p_type: u32);
}

impl<E: Endian> ProgramHeaderExt for ProgramHeader32<E> {
    fn from_generic(endian: Self::Endian, generic: &GenericProgramHeader) -> Self {
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

    fn set_p_type(&mut self, endian: Self::Endian, p_type: u32) {
        self.p_type.set(endian, p_type)
    }
}

impl<E: Endian> ProgramHeaderExt for ProgramHeader64<E> {
    fn from_generic(endian: Self::Endian, generic: &GenericProgramHeader) -> Self {
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

    fn set_p_type(&mut self, endian: Self::Endian, p_type: u32) {
        self.p_type.set(endian, p_type)
    }
}
