//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fmt;

use object::read::elf::{ElfFile, FileHeader};
use object::read::ReadRef;
use object::{Endian, File, Object, ObjectSegment, ObjectSymbol};
use thiserror::Error;

#[derive(Default)]
pub struct Patches {
    patches: Vec<(u64, Vec<u8>)>,
}

impl Patches {
    pub fn new() -> Self {
        Self { patches: vec![] }
    }

    pub fn add_bytes(&mut self, vaddr: u64, value: Vec<u8>) {
        self.patches.push((vaddr, value))
    }

    pub fn add(&mut self, vaddr: u64, value: impl PatchValue, endian: impl Endian) {
        self.add_bytes(vaddr, value.to_bytes(endian))
    }

    pub fn add_bytes_via_symbol<'data, T: FileHeader, R: ReadRef<'data>>(
        &mut self,
        elf_file_for_symbols: &ElfFile<'data, T, R>,
        name: &str,
        value: Vec<u8>,
    ) -> Result<u64, PatchesAddFromSymbolError> {
        for symbol in elf_file_for_symbols.symbols() {
            if symbol.name()? == name {
                let vaddr = symbol.address();
                let size = symbol.size();
                if usize::try_from(size).unwrap() != value.len() {
                    return Err(PatchesAddFromSymbolError::SymbolSizeMismatch(
                        name.to_owned(),
                    ));
                }
                self.add_bytes(vaddr, value);
                return Ok(vaddr);
            }
        }
        Err(PatchesAddFromSymbolError::SymbolMissing(name.to_owned()))
    }

    pub fn add_via_symbol<'data, T: FileHeader, R: ReadRef<'data>>(
        &mut self,
        elf_file_for_symbols: &ElfFile<'data, T, R>,
        name: &str,
        value: impl PatchValue,
        endian: impl Endian,
    ) -> Result<u64, PatchesAddFromSymbolError> {
        self.add_bytes_via_symbol(elf_file_for_symbols, name, value.to_bytes(endian))
    }

    pub fn apply(&self, elf_file_data: &mut [u8]) -> Result<(), PatchesApplyError> {
        let offsets_into_file = match File::parse(&*elf_file_data)? {
            File::Elf32(elf_file) => self.offsets_into_file(&elf_file),
            File::Elf64(elf_file) => self.offsets_into_file(&elf_file),
            _ => {
                panic!()
            }
        }?;

        for (offset_into_file, value) in offsets_into_file
            .iter()
            .zip(self.patches.iter().map(|(_vaddr, value)| value))
        {
            elf_file_data[usize::try_from(*offset_into_file).unwrap()..][..value.len()]
                .copy_from_slice(value);
        }

        Ok(())
    }

    fn offsets_into_file<'data, T: FileHeader, R: ReadRef<'data>>(
        &self,
        elf_file: &ElfFile<'data, T, R>,
    ) -> Result<Vec<u64>, PatchesApplyError> {
        self.patches
            .iter()
            .map(|(vaddr, value)| {
                elf_file
                    .segments()
                    .find_map(|segment| {
                        let start = segment.address();
                        let end = start + segment.size();
                        if (start..end).contains(vaddr) {
                            let offset_in_segment = vaddr - start;
                            let (file_start, file_size) = segment.file_range();
                            if offset_in_segment + u64::try_from(value.len()).unwrap() <= file_size
                            {
                                return Some(file_start + offset_in_segment);
                            }
                        }
                        None
                    })
                    .ok_or(PatchesApplyError::AddrRangeNotMappedWithData(
                        *vaddr,
                        value.len(),
                    ))
            })
            .collect::<Result<Vec<_>, PatchesApplyError>>()
    }
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

#[derive(Error, Debug)]
pub enum PatchesAddFromSymbolError {
    ReadError(object::read::Error),
    SymbolMissing(String),
    SymbolSizeMismatch(String),
}

impl From<object::read::Error> for PatchesAddFromSymbolError {
    fn from(err: object::read::Error) -> Self {
        Self::ReadError(err)
    }
}

impl fmt::Display for PatchesAddFromSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadError(err) => write!(f, "read error: {}", err),
            Self::SymbolMissing(name) => write!(f, "symbol missing: {:?}", name),
            Self::SymbolSizeMismatch(name) => write!(f, "symbol size mismatch: {:?}", name),
        }
    }
}

#[derive(Error, Debug)]
pub enum PatchesApplyError {
    ReadError(object::read::Error),
    AddrRangeNotMappedWithData(u64, usize),
}

impl From<object::read::Error> for PatchesApplyError {
    fn from(err: object::read::Error) -> Self {
        Self::ReadError(err)
    }
}

impl fmt::Display for PatchesApplyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadError(err) => write!(f, "read error: {}", err),
            Self::AddrRangeNotMappedWithData(start, size) => write!(
                f,
                "address range not mapped with file data: {:?}({:?})",
                start, size
            ),
        }
    }
}
