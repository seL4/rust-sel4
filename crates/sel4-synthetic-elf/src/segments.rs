//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::borrow::Cow;
use std::fmt;
use std::ops::Range;

use object::elf::{PF_R, PT_LOAD};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::read::ReadRef;
use object::{Endian, Endianness, Object};
use thiserror::Error;

#[derive(Default)]
pub struct Segments<'a> {
    segments: Vec<Segment<'a>>,
}

impl<'a> Segments<'a> {
    pub fn new() -> Self {
        Self { segments: vec![] }
    }

    pub fn add_segment(&mut self, segment: Segment<'a>) {
        self.segments.push(segment);
    }

    pub fn add_segments_from_phdrs<T: FileHeader, R: ReadRef<'a>>(
        &mut self,
        elf_file: &ElfFile<'a, T, R>,
    ) -> Result<(), SegmentsError> {
        let endian = elf_file.endian();
        for phdr in elf_file.elf_program_headers() {
            if phdr.p_type(endian) == PT_LOAD {
                self.add_segment(Segment::from_phdr(phdr, endian, elf_file.data())?);
            }
        }
        Ok(())
    }

    pub fn footprint(&self) -> Option<Range<u64>> {
        let start = self.segments.iter().map(|segment| segment.p_vaddr).min()?;
        let end = self
            .segments
            .iter()
            .map(|segment| segment.p_vaddr + segment.p_memsz)
            .max()?;
        Some(start..end)
    }

    pub fn build(
        &self,
        endianness: Endianness,
        is_64: bool,
        ehdr: &object::write::elf::FileHeader,
        discard_p_align: bool,
    ) -> Result<Vec<u8>, SegmentsError> {
        let mut writer_buf = vec![];
        let mut writer = object::write::elf::Writer::new(endianness, is_64, &mut writer_buf);

        writer.reserve_file_header();
        writer.reserve_program_headers(self.segments.len().try_into().unwrap());

        let mut offsets = vec![];
        for segment in self.segments.iter() {
            let min_offset: u64 = writer.reserved_len().try_into().unwrap();
            let residue = segment.p_vaddr % segment.p_align;
            let mut offset = min_offset.next_multiple_of(segment.p_align) + residue;
            if offset >= min_offset + segment.p_align {
                offset -= segment.p_align;
            }
            writer.reserve_until(offset.try_into().unwrap());
            writer.reserve(segment.data.len(), 1);
            offsets.push(offset)
        }

        writer.write_file_header(ehdr)?;
        writer.write_align_program_headers();

        for (phdr, offset) in self.segments.iter().zip(offsets.iter()) {
            writer.write_program_header(&object::write::elf::ProgramHeader {
                p_type: PT_LOAD,
                p_flags: phdr.p_flags,
                p_offset: *offset,
                p_vaddr: phdr.p_vaddr,
                p_paddr: phdr.p_paddr,
                p_filesz: phdr.data.len().try_into().unwrap(),
                p_memsz: phdr.p_memsz,
                p_align: if discard_p_align { 1 } else { phdr.p_align },
            });
        }

        for (phdr, offset) in self.segments.iter().zip(offsets.iter()) {
            writer.pad_until((*offset).try_into().unwrap());
            writer.write(&phdr.data);
        }

        Ok(writer_buf)
    }

    pub fn build_using_ehdr<'data, T: FileHeader, R: ReadRef<'data>>(
        &self,
        elf_file: &ElfFile<'data, T, R>,
        discard_p_align: bool,
    ) -> Result<Vec<u8>, SegmentsError> {
        let endian = elf_file.endian();

        let endianness = if endian.is_little_endian() {
            Endianness::Little
        } else {
            Endianness::Big
        };

        let ehdr = elf_file.elf_header();

        let ehdr_for_write = object::write::elf::FileHeader {
            os_abi: ehdr.e_ident().os_abi,
            abi_version: ehdr.e_ident().abi_version,
            e_type: ehdr.e_type(endian),
            e_machine: ehdr.e_machine(endian),
            e_entry: ehdr.e_entry(endian).into(),
            e_flags: ehdr.e_flags(endian),
        };

        self.build(
            endianness,
            elf_file.is_64(),
            &ehdr_for_write,
            discard_p_align,
        )
    }
}

pub struct Segment<'a> {
    pub p_flags: u32,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_memsz: u64,
    pub p_align: u64,
    pub data: Cow<'a, [u8]>,
}

impl<'a> Segment<'a> {
    pub fn from_phdr<T: ProgramHeader, R: ReadRef<'a>>(
        phdr: &T,
        endian: T::Endian,
        data: R,
    ) -> Result<Self, SegmentsError> {
        Ok(Self {
            p_flags: phdr.p_flags(endian),
            p_vaddr: phdr.p_vaddr(endian).into(),
            p_paddr: phdr.p_paddr(endian).into(),
            p_memsz: phdr.p_memsz(endian).into(),
            p_align: phdr.p_align(endian).into(),
            data: Cow::Borrowed(
                phdr.data(endian, data)
                    .map_err(|_| SegmentsError::FileDataError)?,
            ),
        })
    }

    pub fn simple(vaddr: u64, data: Cow<'a, [u8]>) -> Self {
        Self {
            p_flags: PF_R,
            p_vaddr: vaddr,
            p_paddr: vaddr,
            p_memsz: data.len().try_into().unwrap(),
            p_align: 1,
            data,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum SegmentsError {
    ReadError(object::read::Error),
    WriteError(object::write::Error),
    FileDataError,
}

impl From<object::read::Error> for SegmentsError {
    fn from(err: object::read::Error) -> Self {
        Self::ReadError(err)
    }
}

impl From<object::write::Error> for SegmentsError {
    fn from(err: object::write::Error) -> Self {
        Self::WriteError(err)
    }
}

impl fmt::Display for SegmentsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadError(err) => write!(f, "read error: {}", err),
            Self::WriteError(err) => write!(f, "write error: {}", err),
            Self::FileDataError => write!(f, "file data error"),
        }
    }
}
