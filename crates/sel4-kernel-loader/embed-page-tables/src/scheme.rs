//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Range;

use bitfield::{BitMut, BitRange, BitRangeMut};
use quote::ToTokens;

pub trait Scheme {
    type WordPrimitive: ToTokens + fmt::Debug;

    const PAGE_BITS: usize;
    const LEVEL_BITS: usize;
    const NUM_LEVELS: usize;

    const MIN_LEVEL_FOR_LEAF: usize;

    type LeafDescriptor: SchemeLeafDescriptor<Self::WordPrimitive> + fmt::Debug;

    const EMPTY_DESCRIPTOR: Self::WordPrimitive;
    const SYMBOLIC_BRANCH_DESCRIPTOR_OFFSET: Self::WordPrimitive;

    const RUNTIME_SCHEME_IDENT: &'static str;

    type Hepers = SchemeHelpers<Self>;
}

pub trait SchemeLeafDescriptor<WordPrimitive> {
    fn from_paddr(paddr: u64, level: usize) -> Self;

    fn to_raw(&self) -> WordPrimitive;
}

pub struct SchemeHelpers<T: ?Sized>(PhantomData<T>);

impl<T: Scheme> SchemeHelpers<T> {
    pub const fn word_bits() -> usize {
        mem::size_of::<T::WordPrimitive>() * 8
    }

    pub const fn num_entries_in_table() -> usize {
        1 << T::LEVEL_BITS
    }

    pub const fn vaddr_bits() -> usize {
        T::LEVEL_BITS * T::NUM_LEVELS + T::PAGE_BITS
    }

    pub const fn vaddr_mask() -> u64 {
        (1 << Self::vaddr_bits()) - 1
    }

    pub const fn virt_bounds() -> Range<u64> {
        0..(1 << Self::vaddr_bits())
    }

    pub fn largest_leaf_size_bits() -> usize {
        T::LEVEL_BITS * (T::NUM_LEVELS - T::MIN_LEVEL_FOR_LEAF - 1) + T::PAGE_BITS
    }

    pub(crate) fn leaf_descriptor_from_paddr_with_check(
        paddr: u64,
        level: usize,
    ) -> T::LeafDescriptor {
        let num_zero_bits = (T::NUM_LEVELS - level - 1) * T::LEVEL_BITS + T::PAGE_BITS;
        let mask = (1 << num_zero_bits) - 1;
        assert_eq!(paddr & mask, 0);
        T::LeafDescriptor::from_paddr(paddr, level)
    }
}

#[derive(Debug)]
pub enum AArch64 {}

impl Scheme for AArch64 {
    type WordPrimitive = u64;

    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: usize = 9;
    const NUM_LEVELS: usize = 4;

    const MIN_LEVEL_FOR_LEAF: usize = 1;

    type LeafDescriptor = AArch64LeafDescriptor;

    const EMPTY_DESCRIPTOR: Self::WordPrimitive = 0b0;
    const SYMBOLIC_BRANCH_DESCRIPTOR_OFFSET: Self::WordPrimitive = 0b11;

    const RUNTIME_SCHEME_IDENT: &'static str = "AArch64";
}

#[derive(Debug)]
pub struct AArch64LeafDescriptor(u64);

impl SchemeLeafDescriptor<u64> for AArch64LeafDescriptor {
    fn from_paddr(paddr: u64, level: usize) -> Self {
        let mut desc = paddr;
        desc.set_bit_range(1, 0, if level == 3 { 0b11 } else { 0b01 });
        Self(desc)
    }

    fn to_raw(&self) -> u64 {
        self.0
    }
}

impl AArch64LeafDescriptor {
    pub fn set_access_flag(mut self, value: bool) -> Self {
        self.0.set_bit(10, value);
        self
    }

    pub fn set_attribute_index(mut self, index: u64) -> Self {
        assert_eq!(index >> 3, 0);
        self.0.set_bit_range(4, 2, index);
        self
    }

    pub fn set_shareability(mut self, shareability: u64) -> Self {
        assert_eq!(shareability >> 2, 0);
        self.0.set_bit_range(9, 8, shareability);
        self
    }
}

const RISCV_ENCODE_FOR_LINKING_LEFT_ROTATION: u32 = 2;

#[allow(dead_code)]
const fn riscv32_encode_for_linking(word: u32) -> u32 {
    word.rotate_left(RISCV_ENCODE_FOR_LINKING_LEFT_ROTATION)
}

const fn riscv64_encode_for_linking(word: u64) -> u64 {
    word.rotate_left(RISCV_ENCODE_FOR_LINKING_LEFT_ROTATION)
}

#[derive(Debug)]
pub enum Riscv64Sv39 {}

impl Scheme for Riscv64Sv39 {
    type WordPrimitive = u64;

    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: usize = 9;
    const NUM_LEVELS: usize = 3;

    const MIN_LEVEL_FOR_LEAF: usize = 0;

    type LeafDescriptor = Riscv64Sv39LeafDescriptor;

    const EMPTY_DESCRIPTOR: Self::WordPrimitive = riscv64_encode_for_linking(0b0);
    const SYMBOLIC_BRANCH_DESCRIPTOR_OFFSET: Self::WordPrimitive = riscv64_encode_for_linking(0b1);

    const RUNTIME_SCHEME_IDENT: &'static str = "RiscV64";
}

#[derive(Debug)]
pub struct Riscv64Sv39LeafDescriptor(u64);

impl SchemeLeafDescriptor<u64> for Riscv64Sv39LeafDescriptor {
    fn from_paddr(paddr: u64, _level: usize) -> Self {
        let mut desc = 0u64;
        desc.set_bit_range(53, 10, BitRange::<u64>::bit_range(&paddr, 55, 12));
        Self(desc)
            .set_valid(true)
            .set_read(true)
            .set_write(true)
            .set_execute(true)
    }

    fn to_raw(&self) -> u64 {
        riscv64_encode_for_linking(self.0)
    }
}

impl Riscv64Sv39LeafDescriptor {
    pub fn set_valid(mut self, value: bool) -> Self {
        self.0.set_bit(0, value);
        self
    }

    pub fn set_read(mut self, value: bool) -> Self {
        self.0.set_bit(1, value);
        self
    }

    pub fn set_write(mut self, value: bool) -> Self {
        self.0.set_bit(2, value);
        self
    }

    pub fn set_execute(mut self, value: bool) -> Self {
        self.0.set_bit(3, value);
        self
    }
}

#[derive(Debug)]
pub enum Riscv32Sv32 {}

impl Scheme for Riscv32Sv32 {
    type WordPrimitive = u32;

    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: usize = 10;
    const NUM_LEVELS: usize = 2;

    const MIN_LEVEL_FOR_LEAF: usize = 0;

    type LeafDescriptor = Riscv32Sv32LeafDescriptor;

    const EMPTY_DESCRIPTOR: Self::WordPrimitive = riscv32_encode_for_linking(0b0);
    const SYMBOLIC_BRANCH_DESCRIPTOR_OFFSET: Self::WordPrimitive = riscv32_encode_for_linking(0b1);

    const RUNTIME_SCHEME_IDENT: &'static str = "RiscV32";
}

#[derive(Debug)]
pub struct Riscv32Sv32LeafDescriptor(u32);

impl SchemeLeafDescriptor<u32> for Riscv32Sv32LeafDescriptor {
    fn from_paddr(paddr: u64, _level: usize) -> Self {
        let mut desc = 0u32;
        desc.set_bit_range(29, 10, BitRange::<u32>::bit_range(&paddr, 31, 12));
        Self(desc)
            .set_valid(true)
            .set_read(true)
            .set_write(true)
            .set_execute(true)
    }

    fn to_raw(&self) -> u32 {
        riscv32_encode_for_linking(self.0)
    }
}

impl Riscv32Sv32LeafDescriptor {
    pub fn set_valid(mut self, value: bool) -> Self {
        self.0.set_bit(0, value);
        self
    }

    pub fn set_read(mut self, value: bool) -> Self {
        self.0.set_bit(1, value);
        self
    }

    pub fn set_write(mut self, value: bool) -> Self {
        self.0.set_bit(2, value);
        self
    }

    pub fn set_execute(mut self, value: bool) -> Self {
        self.0.set_bit(3, value);
        self
    }
}
