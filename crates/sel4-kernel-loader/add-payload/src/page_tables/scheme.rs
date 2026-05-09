//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

use bitfield::{BitMut, BitRangeMut};

use sel4_config_types::Configuration;

pub type RawDescriptor = u64;

pub type Level = u8;

pub enum Scheme {
    AArch64,
    AArch32,
    RiscVSv39,
    RiscVSv32,
}

impl Scheme {
    pub fn from_config(kernel_config: &Configuration) -> Self {
        let sel4_arch = kernel_config.get("SEL4_ARCH").unwrap().as_str().unwrap();
        let pt_levels = || {
            kernel_config
                .get("PT_LEVELS")
                .unwrap()
                .as_str()
                .unwrap()
                .parse::<u8>()
                .unwrap()
        };
        match sel4_arch {
            "aarch64" => Self::AArch64,
            "aarch32" => Self::AArch32,
            "riscv64" if pt_levels() == 3 => Self::RiscVSv39,
            "riscv32" if pt_levels() == 2 => Self::RiscVSv32,
            _ => panic!("unsupported configuration"),
        }
    }

    pub fn word_bytes(&self) -> usize {
        match self {
            Self::AArch64 | Self::RiscVSv39 => size_of::<u64>(),
            Self::AArch32 | Self::RiscVSv32 => size_of::<u32>(),
        }
    }

    pub fn page_bits(&self) -> u64 {
        12
    }

    pub fn num_levels(&self) -> u8 {
        match self {
            Self::AArch64 => 4,
            Self::AArch32 => 2,
            Self::RiscVSv39 => 3,
            Self::RiscVSv32 => 2,
        }
    }

    pub fn level_bits(&self, level: Level) -> u64 {
        match self {
            Self::AArch64 => 9,
            Self::AArch32 => match level {
                0 => 12,
                1 => 8,
                _ => unreachable!(),
            },
            Self::RiscVSv39 => 9,
            Self::RiscVSv32 => 10,
        }
    }

    pub fn level_align_bits(&self, level: Level) -> u64 {
        self.level_bits(level) + u64::from(self.word_bytes().trailing_zeros())
    }

    pub fn min_level_for_leaf(&self) -> Level {
        match self {
            Self::AArch64 => 1,
            Self::AArch32 => 0,
            Self::RiscVSv39 | Self::RiscVSv32 => 0,
        }
    }

    pub fn empty_descriptor(&self) -> RawDescriptor {
        0
    }

    pub fn branch_descriptor(&self, child_vaddr: u64) -> RawDescriptor {
        match self {
            Self::AArch64 => child_vaddr | 0b11,
            Self::AArch32 => child_vaddr | 0b01,
            Self::RiscVSv39 | Self::RiscVSv32 => (child_vaddr >> 2) | 0b1,
        }
    }

    pub fn descriptor_to_bytes(&self, desc: RawDescriptor) -> Vec<u8> {
        desc.to_le_bytes()[..self.word_bytes()].to_vec()
    }

    pub fn num_entries_in_table(&self, level: Level) -> u64 {
        1 << self.level_bits(level)
    }

    pub fn vaddr_bits(&self) -> u64 {
        (0..self.num_levels())
            .map(|level| self.level_bits(level))
            .sum::<u64>()
            + self.page_bits()
    }

    pub fn vaddr_mask(&self) -> u64 {
        (1 << self.vaddr_bits()) - 1
    }

    pub fn virt_bounds(&self) -> Range<u64> {
        0..(1 << self.vaddr_bits())
    }

    pub fn largest_leaf_size_bits(&self) -> u64 {
        ((self.min_level_for_leaf() + 1)..self.num_levels())
            .map(|level| self.level_bits(level))
            .sum::<u64>()
            + self.page_bits()
    }

    pub fn check_paddr_for_level(&self, level: Level, paddr: u64) {
        let num_zero_bits = ((level + 1)..self.num_levels())
            .map(|level| self.level_bits(level))
            .sum::<u64>()
            + self.page_bits();
        let mask = (1 << num_zero_bits) - 1;
        assert_eq!(paddr & mask, 0);
    }

    pub fn leaf_descriptor_from_level_paddr<T: LeafDescriptor>(
        &self,
        level: Level,
        paddr: u64,
    ) -> T {
        self.check_paddr_for_level(level, paddr);
        T::from_level_paddr(level, paddr)
    }
}

pub trait LeafDescriptor {
    fn from_level_paddr(level: Level, paddr: u64) -> Self;
    fn to_raw(self) -> RawDescriptor;
}

#[derive(Debug)]
pub struct AArch64LeafDescriptor {
    raw: RawDescriptor,
}

impl LeafDescriptor for AArch64LeafDescriptor {
    fn from_level_paddr(level: Level, paddr: u64) -> Self {
        let mut raw = paddr;
        raw.set_bit_range(1, 0, if level == 3 { 0b11 } else { 0b01 });
        Self { raw }
    }

    fn to_raw(self) -> RawDescriptor {
        self.raw
    }
}

impl AArch64LeafDescriptor {
    pub fn set_access_flag(mut self, value: bool) -> Self {
        self.raw.set_bit(10, value);
        self
    }

    pub fn set_attribute_index(mut self, index: u64) -> Self {
        assert_eq!(index >> 3, 0);
        self.raw.set_bit_range(4, 2, index);
        self
    }

    pub fn set_shareability(mut self, shareability: u64) -> Self {
        assert_eq!(shareability >> 2, 0);
        self.raw.set_bit_range(9, 8, shareability);
        self
    }
}

#[derive(Debug)]
pub struct AArch32LeafDescriptor {
    level: Level,
    raw: RawDescriptor,
}

impl LeafDescriptor for AArch32LeafDescriptor {
    fn from_level_paddr(level: Level, paddr: u64) -> Self {
        let mut raw = paddr;
        raw.set_bit_range(1, 0, 0b10);
        Self { level, raw }
    }

    fn to_raw(self) -> RawDescriptor {
        self.raw
    }
}

impl AArch32LeafDescriptor {
    pub fn set_access_flag(mut self, value: bool) -> Self {
        let ix = match self.level {
            0 => 10,
            1 => 4,
            _ => unreachable!(),
        };
        self.raw.set_bit(ix, value);
        self
    }

    pub fn set_attributes(mut self, tex: u32, c: bool, b: bool) -> Self {
        assert_eq!(tex >> 3, 0);
        let (tex_hi, tex_lo) = match self.level {
            0 => (14, 12),
            1 => (8, 6),
            _ => unreachable!(),
        };
        self.raw.set_bit_range(tex_hi, tex_lo, tex);
        self.raw.set_bit(3, c);
        self.raw.set_bit(2, b);
        self
    }

    pub fn set_shareability(mut self, value: bool) -> Self {
        let ix = match self.level {
            0 => 16,
            1 => 10,
            _ => unreachable!(),
        };
        self.raw.set_bit(ix, value);
        self
    }
}

#[derive(Debug)]
pub struct RiscVLeafDescriptor {
    raw: RawDescriptor,
}

impl LeafDescriptor for RiscVLeafDescriptor {
    fn from_level_paddr(_level: Level, paddr: u64) -> Self {
        let raw = paddr >> 2;
        Self { raw }
            .set_valid(true)
            .set_read(true)
            .set_write(true)
            .set_execute(true)
    }

    fn to_raw(self) -> RawDescriptor {
        self.raw
    }
}

impl RiscVLeafDescriptor {
    pub fn set_valid(mut self, value: bool) -> Self {
        self.raw.set_bit(0, value);
        self
    }

    pub fn set_read(mut self, value: bool) -> Self {
        self.raw.set_bit(1, value);
        self
    }

    pub fn set_write(mut self, value: bool) -> Self {
        self.raw.set_bit(2, value);
        self
    }

    pub fn set_execute(mut self, value: bool) -> Self {
        self.raw.set_bit(3, value);
        self
    }
}
