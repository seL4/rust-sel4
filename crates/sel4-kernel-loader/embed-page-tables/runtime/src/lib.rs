//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::cell::UnsafeCell;

#[repr(align(1024))]
pub struct A1024;

#[repr(align(4096))]
pub struct A4096;

#[repr(align(16384))]
pub struct A16384;

#[repr(C)]
pub struct Table<A, const N: usize> {
    _alignment: [A; 0],
    entries: UnsafeCell<[Entry; N]>,
}

unsafe impl<A, const N: usize> Sync for Table<A, N> {}

impl<A, const N: usize> Table<A, N> {
    pub const fn new(entries: [Entry; N]) -> Self {
        Self {
            _alignment: [],
            entries: UnsafeCell::new(entries),
        }
    }

    pub const fn ptr(&self) -> TablePtr {
        TablePtr::new(self.entries.get())
    }
}

#[derive(Copy, Clone)]
pub struct TablePtr {
    ptr: *mut [Entry],
}

unsafe impl Sync for TablePtr {}

impl TablePtr {
    pub const fn new(ptr: *mut [Entry]) -> Self {
        Self { ptr }
    }

    pub const fn value(&self) -> *mut () {
        self.ptr.cast()
    }
}

pub struct TablePtrs<'a> {
    ptrs: &'a [TablePtr],
}

impl<'a> TablePtrs<'a> {
    pub const fn new(ptrs: &'a [TablePtr]) -> Self {
        Self { ptrs }
    }

    pub const fn table(&self, index: usize) -> TablePtr {
        self.ptrs[index]
    }

    pub const fn root(&self) -> TablePtr {
        self.table(0)
    }

    unsafe fn rotate_each_entry_right(&self, n: u32) {
        for table in self.ptrs.iter() {
            for entry in table.ptr.as_mut().unwrap().iter_mut() {
                *entry = entry.rotate_right(n);
            }
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn finish_for_riscv(&self) {
        self.rotate_each_entry_right(RISCV_ROTATE_RIGHT_FOR_FINISH);
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Entry(*const ());

impl Entry {
    pub const fn new(ptr: Option<*mut ()>, offset: usize) -> Self {
        Self(match ptr {
            Some(ptr) => unsafe { ptr.byte_add(offset) },
            None => offset as *mut (),
        })
    }

    fn rotate_right(self, n: u32) -> Self {
        Self((self.0 as usize).rotate_right(n) as *mut ())
    }
}

const RISCV_ROTATE_RIGHT_FOR_FINISH: u32 = 2;
