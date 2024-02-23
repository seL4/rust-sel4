//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// NOTE
//
// - aarch64 and arm use variant 1 defined in [1][2].
// - x86_64 uses variant 2 defined in [1][2].
// - riscv uses variant 1 with a twist: the thread pointer points to the first address _past_ the
//   TCB [3].
//
// [1] https://akkadia.org/drepper/tls.pdf
// [2] https://fuchsia.dev/fuchsia-src/development/kernel/threads/tls#implementation
// [3] https://github.com/riscv-non-isa/riscv-elf-psabi-doc/blob/master/riscv-elf.adoc#thread-local-storage

#![no_std]

use core::slice;

#[cfg(feature = "on-stack")]
mod on_stack;

#[cfg(feature = "on-stack")]
pub use on_stack::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncheckedTlsImage {
    pub vaddr: usize,
    pub filesz: usize,
    pub memsz: usize,
    pub align: usize,
}

impl UncheckedTlsImage {
    pub fn check(&self) -> Option<TlsImage> {
        if self.memsz >= self.filesz && self.align.is_power_of_two() && self.align > 0 {
            Some(TlsImage {
                vaddr: self.vaddr,
                filesz: self.filesz,
                memsz: self.memsz,
                align: self.align,
            })
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsImage {
    vaddr: usize,
    filesz: usize,
    memsz: usize,
    align: usize,
}

impl TlsImage {
    unsafe fn initialize(&self, tls_base_addr: usize) {
        let image_data_window = slice::from_raw_parts(self.vaddr as *mut u8, self.filesz);
        let tls_window = slice::from_raw_parts_mut(tls_base_addr as *mut u8, self.memsz);
        let (tdata, tbss) = tls_window.split_at_mut(self.filesz);
        tdata.copy_from_slice(image_data_window);
        tbss.fill(0);
    }
}
