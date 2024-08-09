//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::any;
use std::error::Error;
use std::fmt;
use std::ops::Range;

use num::{NumCast, ToPrimitive};
use object::read::ReadRef;

pub use object;
pub use object::elf::{PF_R, PF_W, PF_X};

mod patches;
mod segments;

pub use patches::{PatchValue, Patches};
pub use segments::{Segment, Segments};

pub struct Builder<'a, 'data, T: object::read::elf::FileHeader, R: ReadRef<'data>> {
    base_elf_file: &'a object::read::elf::ElfFile<'data, T, R>,
    segments: Segments<'data>,
    patches: Patches,
    discard_p_align: bool,
}

impl<'a: 'data, 'data, T: object::read::elf::FileHeader, R: ReadRef<'data>>
    Builder<'a, 'data, T, R>
{
    pub fn empty(base_elf_file: &'a object::read::elf::ElfFile<'data, T, R>) -> Self {
        let segments = Segments::new();
        let patches = Patches::new();
        Self {
            base_elf_file,
            segments,
            patches,
            discard_p_align: false,
        }
    }

    pub fn new(
        base_elf_file: &'a object::read::elf::ElfFile<'data, T, R>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut this = Self::empty(base_elf_file);
        this.segments.add_segments_from_phdrs(this.base_elf_file)?;
        Ok(this)
    }

    pub fn discard_p_align(&mut self, doit: bool) {
        self.discard_p_align = doit;
    }

    pub fn add_segment(&mut self, segment: Segment<'a>) {
        self.segments.add_segment(segment)
    }

    pub fn footprint(&self) -> Option<Range<u64>> {
        self.segments.footprint()
    }

    pub fn next_vaddr(&self) -> u64 {
        self.footprint().map(|footprint| footprint.end).unwrap_or(0)
    }

    pub fn patch_bytes(&mut self, name: &str, value: Vec<u8>) -> Result<u64, Box<dyn Error>> {
        Ok(self
            .patches
            .add_bytes_via_symbol(self.base_elf_file, name, value)?)
    }

    pub fn patch(&mut self, name: &str, value: impl PatchValue) -> Result<u64, Box<dyn Error>> {
        Ok(self.patches.add_via_symbol(
            self.base_elf_file,
            name,
            value,
            self.base_elf_file.endian(),
        )?)
    }

    pub fn patch_word(&mut self, name: &str, value: T::Word) -> Result<u64, Box<dyn Error>>
    where
        T::Word: PatchValue,
    {
        self.patch(name, value)
    }

    pub fn patch_word_with_cast(
        &mut self,
        name: &str,
        value: impl ToPrimitive + fmt::Debug + Copy,
    ) -> Result<u64, Box<dyn Error>>
    where
        T::Word: PatchValue + NumCast,
    {
        self.patch(
            name,
            <T::Word as NumCast>::from(value).unwrap_or_else(|| {
                panic!(
                    "value {:#x?} out of bounds for word type {}",
                    value,
                    any::type_name::<T::Word>()
                )
            }),
        )
    }

    pub fn build(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf = self
            .segments
            .build_using_ehdr(self.base_elf_file, self.discard_p_align)?;
        self.patches.apply(&mut buf).unwrap();
        Ok(buf)
    }
}
