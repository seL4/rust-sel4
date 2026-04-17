//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;

use anyhow::{Error, bail};
use object::elf::{PT_NOTE, PT_NULL, ProgramHeader32, ProgramHeader64};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endian, Pod, U32, U64, pod};

pub struct Patching<'a, T: FileHeader> {
    orig_elf: &'a ElfFile<'a, T>,
    data: Vec<u8>,
}

impl<'a, T: FileHeader<Word: Pod, ProgramHeader: ProgramHeaderExt>> Patching<'a, T> {
    pub fn new(orig_elf: &'a ElfFile<'a, T>) -> Self {
        let mut this = Self {
            orig_elf,
            data: orig_elf.data().to_vec(),
        };
        this.clear_placeholder_phdrs();
        this
    }

    pub fn orig_elf(&self) -> &'a ElfFile<'a, T> {
        self.orig_elf
    }

    pub fn endian(&self) -> T::Endian {
        self.orig_elf().endian()
    }

    fn phdrs_range(&self) -> Range<usize> {
        let ehdr = self.orig_elf().elf_header();
        let phoff: usize = ehdr.e_phoff(self.endian()).into().try_into().unwrap();
        let phnum: usize = ehdr.e_phnum(self.endian()).into();
        let phentsize: usize = ehdr.e_phentsize(self.endian()).into();
        phoff..(phoff + phnum * phentsize)
    }

    fn phdrs(&self) -> &[T::ProgramHeader] {
        let range = self.phdrs_range();
        pod::slice_from_all_bytes(&self.data[range]).unwrap()
    }

    fn phdrs_mut(&mut self) -> &mut [T::ProgramHeader] {
        let range = self.phdrs_range();
        pod::slice_from_all_bytes_mut(&mut self.data[range]).unwrap()
    }

    fn clear_placeholder_phdrs(&mut self) {
        let endian = self.endian();
        for phdr_slot in self.phdrs_mut().iter_mut() {
            if phdr_slot.p_type(endian) == PT_NOTE && phdr_slot.p_filesz(endian).into() == 0 {
                pod::bytes_of_mut(phdr_slot).fill(0);
                phdr_slot.set_p_type(endian, PT_NULL);
            }
        }
    }

    pub fn add_phdr(&mut self, phdr: T::ProgramHeader) -> Result<&mut T::ProgramHeader, Error> {
        let endian = self.endian();
        for phdr_slot in self.phdrs_mut().iter_mut() {
            if phdr_slot.p_type(endian) == PT_NULL {
                *phdr_slot = phdr;
                return Ok(phdr_slot);
            }
        }
        bail!("no placeholder phdrs")
    }

    fn footprint(&self) -> Option<Range<u64>> {
        let start = self
            .phdrs()
            .iter()
            .map(|phdr| phdr.p_vaddr(self.endian()).into())
            .min()?;
        let end = self
            .phdrs()
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
    ) -> Result<&T::ProgramHeader, Error> {
        assert!(data_align <= phdr.p_align);
        self.align_data_cursor(data_align);
        phdr.p_offset = self.data.len().try_into().unwrap();
        phdr.p_filesz = data.len().try_into().unwrap();
        self.data.extend_from_slice(data);
        self.add_segment_raw(phdr)
    }

    pub fn add_segment_raw(
        &mut self,
        mut phdr: GenericProgramHeader,
    ) -> Result<&T::ProgramHeader, Error> {
        let p_vaddr = self.next_aligned_vaddr(phdr.p_align) + phdr.p_offset % phdr.p_align.max(1);
        phdr.p_vaddr = p_vaddr;
        phdr.p_paddr = p_vaddr;
        Ok(&*self.add_phdr(T::ProgramHeader::from_generic(self.endian(), &phdr))?)
    }

    pub fn finalize(self) -> Vec<u8> {
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

#[allow(dead_code)]
pub trait ProgramHeaderExt: ProgramHeader {
    fn from_generic(endian: Self::Endian, generic: &GenericProgramHeader) -> Self;
    fn set_p_type(&mut self, endian: Self::Endian, p_type: u32);
    fn set_p_flags(&mut self, endian: Self::Endian, p_flags: u32);
    fn set_p_offset(&mut self, endian: Self::Endian, p_offset: Self::Word);
    fn set_p_vaddr(&mut self, endian: Self::Endian, p_vaddr: Self::Word);
    fn set_p_paddr(&mut self, endian: Self::Endian, p_paddr: Self::Word);
    fn set_p_filesz(&mut self, endian: Self::Endian, p_filesz: Self::Word);
    fn set_p_memsz(&mut self, endian: Self::Endian, p_memsz: Self::Word);
    fn set_p_align(&mut self, endian: Self::Endian, p_align: Self::Word);
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

    fn set_p_flags(&mut self, endian: Self::Endian, p_flags: u32) {
        self.p_flags.set(endian, p_flags)
    }

    fn set_p_offset(&mut self, endian: Self::Endian, p_offset: Self::Word) {
        self.p_offset.set(endian, p_offset)
    }

    fn set_p_vaddr(&mut self, endian: Self::Endian, p_vaddr: Self::Word) {
        self.p_vaddr.set(endian, p_vaddr)
    }

    fn set_p_paddr(&mut self, endian: Self::Endian, p_paddr: Self::Word) {
        self.p_vaddr.set(endian, p_paddr)
    }

    fn set_p_filesz(&mut self, endian: Self::Endian, p_filesz: Self::Word) {
        self.p_offset.set(endian, p_filesz)
    }

    fn set_p_memsz(&mut self, endian: Self::Endian, p_memsz: Self::Word) {
        self.p_vaddr.set(endian, p_memsz)
    }

    fn set_p_align(&mut self, endian: Self::Endian, p_align: Self::Word) {
        self.p_vaddr.set(endian, p_align)
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

    fn set_p_flags(&mut self, endian: Self::Endian, p_flags: u32) {
        self.p_flags.set(endian, p_flags)
    }

    fn set_p_offset(&mut self, endian: Self::Endian, p_offset: Self::Word) {
        self.p_offset.set(endian, p_offset)
    }

    fn set_p_vaddr(&mut self, endian: Self::Endian, p_vaddr: Self::Word) {
        self.p_vaddr.set(endian, p_vaddr)
    }

    fn set_p_paddr(&mut self, endian: Self::Endian, p_paddr: Self::Word) {
        self.p_vaddr.set(endian, p_paddr)
    }

    fn set_p_filesz(&mut self, endian: Self::Endian, p_filesz: Self::Word) {
        self.p_offset.set(endian, p_filesz)
    }

    fn set_p_memsz(&mut self, endian: Self::Endian, p_memsz: Self::Word) {
        self.p_vaddr.set(endian, p_memsz)
    }

    fn set_p_align(&mut self, endian: Self::Endian, p_align: Self::Word) {
        self.p_vaddr.set(endian, p_align)
    }
}
