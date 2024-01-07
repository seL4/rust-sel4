//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

#![cfg_attr(not(feature = "unstable"), allow(dead_code))]
#![cfg_attr(not(feature = "unstable"), allow(unused_imports))]

use core::mem;

#[cfg(feature = "unstable")]
use core::intrinsics;

use volatile::ops::{Ops, UnitaryOps};

#[cfg(feature = "unstable")]
use volatile::ops::BulkOps;

#[derive(Debug, Default, Copy, Clone)]
pub struct UnorderedAtomicOps;

impl Ops for UnorderedAtomicOps {}

#[cfg(feature = "unstable")]
impl<T: UnsignedPrimitiveWithUnorderedAtomics + Copy> UnitaryOps<T> for UnorderedAtomicOps {
    unsafe fn read(src: *const T) -> T {
        unsafe { intrinsics::atomic_load_unordered(src) }
    }

    unsafe fn write(dst: *mut T, src: T) {
        unsafe { intrinsics::atomic_store_unordered(dst, src) }
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait UnsignedPrimitiveWithUnorderedAtomics {}

macro_rules! impl_unsigned_primitive_with_unordered_atomics {
    (
        $prim:path,
        $target_has_atomic_key:literal,
        $memmove:ident,
        $memcpy:ident,
        $memset:ident,
    ) => {
        #[cfg(target_has_atomic = $target_has_atomic_key)]
        unsafe impl UnsignedPrimitiveWithUnorderedAtomics for $prim {}

        #[cfg(feature = "unstable")]
        #[cfg(target_has_atomic = $target_has_atomic_key)]
        impl BulkOps<$prim> for UnorderedAtomicOps {
            unsafe fn memmove(dst: *mut $prim, src: *const $prim, count: usize) {
                unsafe { $memmove(dst, src, count * mem::size_of::<$prim>()) }
            }

            unsafe fn memcpy(dst: *mut $prim, src: *const $prim, count: usize) {
                unsafe { $memcpy(dst, src, count * mem::size_of::<$prim>()) }
            }

            unsafe fn memset(dst: *mut $prim, val: u8, count: usize) {
                unsafe { $memset(dst, val, count * mem::size_of::<$prim>()) }
            }
        }

        extern "C" {
            fn $memmove(dest: *mut $prim, src: *const $prim, bytes: usize);
            fn $memcpy(dest: *mut $prim, src: *const $prim, bytes: usize);
            fn $memset(s: *mut $prim, c: u8, bytes: usize);
        }
    };
}

impl_unsigned_primitive_with_unordered_atomics! {
    u8,
    "8",
    __llvm_memmove_element_unordered_atomic_1,
    __llvm_memcpy_element_unordered_atomic_1,
    __llvm_memset_element_unordered_atomic_1,
}

impl_unsigned_primitive_with_unordered_atomics! {
    u16,
    "16",
    __llvm_memmove_element_unordered_atomic_2,
    __llvm_memcpy_element_unordered_atomic_2,
    __llvm_memset_element_unordered_atomic_2,
}

impl_unsigned_primitive_with_unordered_atomics! {
    u32,
    "32",
    __llvm_memmove_element_unordered_atomic_4,
    __llvm_memcpy_element_unordered_atomic_4,
    __llvm_memset_element_unordered_atomic_4,
}

impl_unsigned_primitive_with_unordered_atomics! {
    u64,
    "64",
    __llvm_memmove_element_unordered_atomic_8,
    __llvm_memcpy_element_unordered_atomic_8,
    __llvm_memset_element_unordered_atomic_8,
}

#[cfg(target_pointer_width = "32")]
type UsizeCastTarget = u32;

#[cfg(target_pointer_width = "64")]
type UsizeCastTarget = u64;

#[cfg(feature = "unstable")]
#[cfg(any(
    all(target_pointer_width = "32", target_has_atomic = "32"),
    all(target_pointer_width = "64", target_has_atomic = "64"),
))]
impl BulkOps<usize> for UnorderedAtomicOps {
    unsafe fn memmove(dst: *mut usize, src: *const usize, count: usize) {
        unsafe {
            <UnorderedAtomicOps as BulkOps<UsizeCastTarget>>::memmove(dst.cast(), src.cast(), count)
        }
    }

    unsafe fn memcpy(dst: *mut usize, src: *const usize, count: usize) {
        unsafe {
            <UnorderedAtomicOps as BulkOps<UsizeCastTarget>>::memcpy(dst.cast(), src.cast(), count)
        }
    }

    unsafe fn memset(dst: *mut usize, val: u8, count: usize) {
        unsafe { <UnorderedAtomicOps as BulkOps<UsizeCastTarget>>::memset(dst.cast(), val, count) }
    }
}
