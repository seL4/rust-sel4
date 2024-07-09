//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::cell::UnsafeCell;

#[repr(C)]
#[cfg_attr(
    any(
        target_arch = "aarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "x86_64",
    ),
    repr(align(16))
)]
#[cfg_attr(target_arch = "arm", repr(align(4)))]
pub struct Stack<const N: usize>(UnsafeCell<[u8; N]>);

unsafe impl<const N: usize> Sync for Stack<N> {}

impl<const N: usize> Stack<N> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new([0; N]))
    }

    pub const fn bottom(&self) -> StackBottom {
        StackBottom(self.0.get().cast::<u8>().wrapping_add(N))
    }
}

impl<const N: usize> Default for Stack<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
pub struct StackBottom(#[allow(dead_code)] *mut u8);

impl StackBottom {
    pub fn ptr(&self) -> *mut u8 {
        self.0
    }
}

unsafe impl Sync for StackBottom {}
