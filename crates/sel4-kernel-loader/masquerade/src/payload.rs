//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;
use core::slice;

use crate::caches::{clean_dcache_range, invalidate_icache_all};

pub(crate) struct Payload {
    entry: usize,
    regions: &'static [Region],
    data: *const u8,
}

#[repr(C)]
struct Region {
    vaddr: usize,
    offset: usize,
    filesz: usize,
    memsz: usize,
}

impl Payload {
    pub(crate) unsafe fn deserialize(start: *const u8) -> Self {
        unsafe {
            let p = start.cast::<usize>();
            let (&entry, p) = deserialize(p);
            let (&num_regions, p) = deserialize(p);
            let (regions, p) = deserialize_slice(p, num_regions);
            let data = p;
            Self {
                entry,
                regions,
                data,
            }
        }
    }

    pub(crate) unsafe fn deploy(self) -> usize {
        for region in self.regions {
            let vaddr = region.vaddr as *mut u8;
            let filesz = region.filesz;
            let memsz = region.memsz;
            let src = self.data.wrapping_add(region.offset);
            unsafe {
                ptr::copy(src, vaddr, filesz);
                ptr::write_bytes(vaddr.wrapping_add(filesz), 0, memsz - filesz);
                clean_dcache_range(vaddr.addr(), memsz);
            }
        }
        unsafe {
            invalidate_icache_all();
        }
        self.entry
    }
}

unsafe fn deserialize<T, U>(cursor: *const T) -> (&'static T, *const U) {
    unsafe { (&*cursor, cursor.wrapping_add(1).cast()) }
}

unsafe fn deserialize_slice<T, U>(cursor: *const T, n: usize) -> (&'static [T], *const U) {
    unsafe {
        (
            slice::from_raw_parts(cursor, n),
            cursor.wrapping_add(n).cast(),
        )
    }
}
