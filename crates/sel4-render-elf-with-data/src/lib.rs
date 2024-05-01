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

pub trait FileHeaderExt:
    FileHeader + FileHeader<Word: PrimInt, Sword: PrimInt, Endian = Endianness>
{
    fn checked_add_signed(x: Self::Word, y: Self::Sword) -> Option<Self::Word>;
    fn write_word_bytes(endian: impl Endian, n: Self::Word) -> Vec<u8>;
}

impl FileHeaderExt for ConcreteFileHeader32 {
    fn checked_add_signed(x: Self::Word, y: Self::Sword) -> Option<Self::Word> {
        x.checked_add_signed(y)
    }

    fn write_word_bytes(endian: impl Endian, n: Self::Word) -> Vec<u8> {
        endian.write_u32_bytes(n).to_vec()
    }
}

impl FileHeaderExt for ConcreteFileHeader64 {
    fn checked_add_signed(x: Self::Word, y: Self::Sword) -> Option<Self::Word> {
        x.checked_add_signed(y)
    }

    fn write_word_bytes(endian: impl Endian, n: Self::Word) -> Vec<u8> {
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

type ConcreteValue<T> = <T as FileHeader>::Word;

pub struct SymbolicInjection<'a, T: FileHeaderExt> {
    pub align_modulus: T::Word,
    pub align_residue: T::Word,
    pub content: &'a [u8],
    pub memsz: T::Word,
    pub patches: Vec<(Symbol, SymbolicValue<T>)>,
}

#[derive(Debug)]
pub struct SymbolicValue<T: FileHeaderExt> {
    pub addend: T::Sword,
}

impl<'a, T: FileHeaderExt> SymbolicInjection<'a, T> {
    fn filesz(&self) -> T::Word {
        NumCast::from(self.content.len()).unwrap()
    }

    fn align_from(&self, addr: T::Word) -> T::Word {
        align_from::<T>(addr, self.align_modulus, self.align_residue)
    }

    fn locate(&self, vaddr: T::Word) -> Result<Injection<'a, T>> {
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
    pub vaddr: T::Word,
    pub content: &'a [u8],
    pub memsz: T::Word,
    pub patches: Vec<(Symbol, ConcreteValue<T>)>,
}

impl<'a, T: FileHeaderExt> Injection<'a, T> {
    fn vaddr(&self) -> T::Word {
        self.vaddr
    }

    fn filesz(&self) -> T::Word {
        NumCast::from(self.content.len()).unwrap()
    }

    fn memsz(&self) -> T::Word {
        self.memsz
    }

    fn content(&self) -> &'a [u8] {
        self.content
    }

    fn patches(&self) -> impl Iterator<Item = &(Symbol, ConcreteValue<T>)> {
        self.patches.iter()
    }
}

fn align_from<T: FileHeaderExt>(addr: T::Word, modulus: T::Word, residue: T::Word) -> T::Word {
    addr + (modulus + residue - addr % modulus) % modulus
}
