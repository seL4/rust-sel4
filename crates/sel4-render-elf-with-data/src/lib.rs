//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use anyhow::{bail, Result};
use num::{NumCast, PrimInt};
use object::{
    elf::{FileHeader32, FileHeader64},
    read::elf::FileHeader,
    Endian, Endianness, File,
};

mod render;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ElfBitWidth {
    Elf32,
    Elf64,
}

impl ElfBitWidth {
    pub fn detect(file_content: &[u8]) -> Result<Self> {
        Ok(match File::parse(file_content)? {
            File::Elf32(_) => Self::Elf32,
            File::Elf64(_) => Self::Elf64,
            _ => bail!("not ELF"),
        })
    }
}

pub type ConcreteFileHeader32 = FileHeader32<Endianness>;
pub type ConcreteFileHeader64 = FileHeader64<Endianness>;

// NOTE(rustc_wishlist)
//
// This is much simpler with #![feature(associated_type_bounds)]:
// ```
// pub trait FileHeaderExt:
//   FileHeader<Word: PrimInt, Sword: PrimInt, Endian = Endianness>
// ```
//
pub trait FileHeaderExt:
    FileHeader<Word = Self::ExtWord, Sword = Self::ExtSword, Endian = Endianness>
{
    type ExtWord: PrimInt + Into<u64>;
    type ExtSword: PrimInt + Into<i64>;

    fn checked_add_signed(x: Self::ExtWord, y: Self::ExtSword) -> Option<Self::ExtWord>;
    fn write_word_bytes(endian: impl Endian, n: Self::ExtWord) -> Vec<u8>;
}

impl FileHeaderExt for ConcreteFileHeader32 {
    type ExtWord = u32;
    type ExtSword = i32;

    fn checked_add_signed(x: Self::ExtWord, y: Self::ExtSword) -> Option<Self::ExtWord> {
        x.checked_add_signed(y)
    }

    fn write_word_bytes(endian: impl Endian, n: Self::ExtWord) -> Vec<u8> {
        endian.write_u32_bytes(n).to_vec()
    }
}

impl FileHeaderExt for ConcreteFileHeader64 {
    type ExtWord = u64;
    type ExtSword = i64;

    fn checked_add_signed(x: Self::ExtWord, y: Self::ExtSword) -> Option<Self::ExtWord> {
        x.checked_add_signed(y)
    }

    fn write_word_bytes(endian: impl Endian, n: Self::ExtWord) -> Vec<u8> {
        endian.write_u64_bytes(n).to_vec()
    }
}

// NOTE
// The phdrs in output of render_with_data have p_align=1 regardless of the input.
// That is because the current consumers of the output do not use p_align.

pub struct Input<'a, T: FileHeaderExt> {
    pub symbolic_injections: Vec<SymbolicInjection<'a, T>>,
    pub image_start_patches: Vec<Symbol>,
    pub image_end_patches: Vec<Symbol>,
    pub concrete_patches: Vec<(Symbol, ConcreteValue<T>)>,
}

impl<'a, T: FileHeaderExt> Default for Input<'a, T> {
    fn default() -> Self {
        Self {
            symbolic_injections: Default::default(),
            image_start_patches: Default::default(),
            image_end_patches: Default::default(),
            concrete_patches: Default::default(),
        }
    }
}

type Symbol = String;

type ConcreteValue<T> = <T as FileHeaderExt>::ExtWord;

pub struct SymbolicInjection<'a, T: FileHeaderExt> {
    pub align_modulus: T::ExtWord,
    pub align_residue: T::ExtWord,
    pub content: &'a [u8],
    pub memsz: T::ExtWord,
    pub patches: Vec<(Symbol, SymbolicValue<T>)>,
}

#[derive(Debug)]
pub struct SymbolicValue<T: FileHeaderExt> {
    pub addend: T::ExtSword,
}

impl<'a, T: FileHeaderExt> SymbolicInjection<'a, T> {
    fn filesz(&self) -> T::ExtWord {
        NumCast::from(self.content.len()).unwrap()
    }

    fn align_from(&self, addr: T::ExtWord) -> T::ExtWord {
        align_from::<T>(addr, self.align_modulus, self.align_residue)
    }

    fn locate(&self, vaddr: T::ExtWord) -> Result<Injection<'a, T>> {
        Ok(Injection {
            vaddr,
            content: self.content,
            memsz: self.memsz,
            patches: self
                .patches
                .iter()
                .map(|(name, symbolic_value)| {
                    (
                        name.clone(),
                        T::checked_add_signed(vaddr, symbolic_value.addend).unwrap(),
                    )
                })
                .collect::<Vec<(Symbol, ConcreteValue<T>)>>(),
        })
    }
}

pub struct Injection<'a, T: FileHeaderExt> {
    pub vaddr: T::ExtWord,
    pub content: &'a [u8],
    pub memsz: T::ExtWord,
    pub patches: Vec<(Symbol, ConcreteValue<T>)>,
}

impl<'a, T: FileHeaderExt> Injection<'a, T> {
    fn vaddr(&self) -> T::ExtWord {
        self.vaddr
    }

    fn filesz(&self) -> T::ExtWord {
        NumCast::from(self.content.len()).unwrap()
    }

    fn memsz(&self) -> T::ExtWord {
        self.memsz
    }

    fn content(&self) -> &'a [u8] {
        self.content
    }

    fn patches(&self) -> impl Iterator<Item = &(Symbol, ConcreteValue<T>)> {
        self.patches.iter()
    }
}

fn align_from<T: FileHeaderExt>(
    addr: T::ExtWord,
    modulus: T::ExtWord,
    residue: T::ExtWord,
) -> T::ExtWord {
    addr + (modulus + residue - addr % modulus) % modulus
}
