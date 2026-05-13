//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use object::elf::{FileHeader32, FileHeader64};
use object::{Endianness, File};

use super as low_level;

pub enum Patching<'a> {
    Patching32(low_level::Patching<'a, FileHeader32<Endianness>>),
    Patching64(low_level::Patching<'a, FileHeader64<Endianness>>),
}

impl<'a> Patching<'a> {
    pub fn new(file: &'a File) -> Self {
        match file {
            File::Elf32(elf) => Self::Patching32(low_level::Patching::new(elf)),
            File::Elf64(elf) => Self::Patching64(low_level::Patching::new(elf)),
            _ => panic!("unsupported format"),
        }
    }

    pub fn patch_symbol(&mut self, symbol_name: &str, value: &[u8]) {
        match self {
            Self::Patching32(this) => this.patch_symbol(symbol_name, value),
            Self::Patching64(this) => this.patch_symbol(symbol_name, value),
        }
    }

    pub fn patch_word(&mut self, symbol_name: &str, word: u64) {
        match self {
            Self::Patching32(this) => this.patch_word(symbol_name, word.try_into().unwrap()),
            Self::Patching64(this) => this.patch_word(symbol_name, word),
        }
    }

    pub fn add_data_segment<D: AsRef<[u8]>>(&mut self, data_align: u64, f: impl FnOnce(u64) -> D) {
        match self {
            Self::Patching32(this) => this.add_data_segment(data_align, f),
            Self::Patching64(this) => this.add_data_segment(data_align, f),
        }
    }

    pub fn add_data_segment_with_meta_phdr(&mut self, p_type: u32, data_align: u64, data: &[u8]) {
        match self {
            Self::Patching32(this) => {
                this.add_data_segment_with_meta_phdr(p_type, data_align, data)
            }
            Self::Patching64(this) => {
                this.add_data_segment_with_meta_phdr(p_type, data_align, data)
            }
        }
    }

    pub fn finalize(self) -> Vec<u8> {
        match self {
            Self::Patching32(this) => this.finalize(),
            Self::Patching64(this) => this.finalize(),
        }
    }
}
