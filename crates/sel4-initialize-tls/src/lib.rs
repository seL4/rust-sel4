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

use core::alloc::Layout;
use core::mem;
use core::slice;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "x86_64",
)))]
compile_error!("unsupported architecture");

mod set_thread_pointer;

pub use set_thread_pointer::{SetThreadPointerFn, DEFAULT_SET_THREAD_POINTER_FN};

#[cfg(feature = "on-stack")]
mod on_stack;

#[cfg(feature = "on-stack")]
pub use on_stack::*;

#[cfg(feature = "on-heap")]
mod on_heap;

#[cfg(feature = "on-heap")]
pub use on_heap::*;

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
    pub fn reservation_layout(&self) -> TlsReservationLayout {
        TlsReservationLayout::from_segment_layout(self.segment_layout())
    }

    fn segment_layout(&self) -> Layout {
        Layout::from_size_align(self.memsz, self.align).unwrap()
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn initialize_tls_reservation(&self, tls_reservation_start: *mut u8) {
        let reservation_layout = self.reservation_layout();

        let image_data_window = slice::from_raw_parts(self.vaddr as *mut u8, self.filesz);

        let segment_start =
            tls_reservation_start.wrapping_byte_add(reservation_layout.segment_offset());
        let segment_window = slice::from_raw_parts_mut(segment_start, self.memsz);
        let (tdata, tbss) = segment_window.split_at_mut(self.filesz);

        tdata.copy_from_slice(image_data_window);
        tbss.fill(0);

        if cfg!(target_arch = "x86_64") {
            let thread_pointer =
                tls_reservation_start.wrapping_byte_add(reservation_layout.thread_pointer_offset());
            (thread_pointer.cast::<*mut u8>()).write(thread_pointer);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TlsReservationLayout {
    footprint: Layout,
    segment_offset: usize,
    thread_pointer_offset: usize,
}

impl TlsReservationLayout {
    fn from_segment_layout(segment_layout: Layout) -> Self {
        if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
            let tcb_size = 2 * mem::size_of::<usize>();
            let segment_offset = tcb_size.next_multiple_of(segment_layout.align());
            Self {
                footprint: Layout::from_size_align(
                    segment_offset + segment_layout.size(),
                    segment_layout.align().max(tcb_size),
                )
                .unwrap(),
                segment_offset,
                thread_pointer_offset: 0,
            }
        } else if cfg!(any(target_arch = "riscv32", target_arch = "riscv64")) {
            Self {
                footprint: Layout::from_size_align(segment_layout.size(), segment_layout.align())
                    .unwrap(),
                segment_offset: 0,
                thread_pointer_offset: 0,
            }
        } else if cfg!(target_arch = "x86_64") {
            let tcb_size = 2 * mem::size_of::<usize>(); // could probably get away with just 1x word size
            let thread_pointer_offset = segment_layout
                .size()
                .next_multiple_of(segment_layout.align());
            Self {
                footprint: Layout::from_size_align(
                    thread_pointer_offset + tcb_size,
                    segment_layout.align(),
                )
                .unwrap(),
                segment_offset: 0,
                thread_pointer_offset,
            }
        } else {
            unreachable!();
        }
    }

    pub fn footprint(&self) -> Layout {
        self.footprint
    }

    pub fn segment_offset(&self) -> usize {
        self.segment_offset
    }

    pub fn thread_pointer_offset(&self) -> usize {
        self.thread_pointer_offset
    }
}
