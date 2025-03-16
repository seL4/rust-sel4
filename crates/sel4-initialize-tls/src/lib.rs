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
use core::ptr;
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

mod static_allocation;
pub use static_allocation::*;

#[cfg(feature = "on-stack")]
mod on_stack;

#[cfg(feature = "on-heap")]
mod on_heap;

#[cfg(feature = "on-heap")]
pub use on_heap::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UncheckedTlsImage {
    pub vaddr: usize,
    pub filesz: usize,
    pub memsz: usize,
    pub align: usize,
}

impl UncheckedTlsImage {
    pub fn check(&self) -> Result<TlsImage, InvalidTlsImageError> {
        if self.memsz >= self.filesz && self.align.is_power_of_two() && self.align > 0 {
            Ok(TlsImage { checked: *self })
        } else {
            Err(InvalidTlsImageError::new())
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidTlsImageError(());

impl InvalidTlsImageError {
    fn new() -> Self {
        Self(())
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TlsImage {
    checked: UncheckedTlsImage,
}

impl TlsImage {
    pub fn reservation_layout(&self) -> TlsReservationLayout {
        TlsReservationLayout::from_segment_layout(self.segment_layout())
    }

    fn segment_layout(&self) -> Layout {
        Layout::from_size_align(self.checked.memsz, self.checked.align).unwrap()
    }

    fn image_data(&self) -> *const [u8] {
        ptr::slice_from_raw_parts(self.checked.vaddr as *mut u8, self.checked.filesz)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn initialize_reservation(&self, reservation_start: *mut u8) -> usize {
        let reservation_layout = self.reservation_layout();
        let reservation =
            slice::from_raw_parts_mut(reservation_start, reservation_layout.footprint().size());
        let (tdata, tbss) = reservation[reservation_layout.segment_offset()..]
            [..self.checked.memsz]
            .split_at_mut(self.checked.filesz);
        tdata.copy_from_slice(self.image_data().as_ref().unwrap());
        tbss.fill(0);
        let thread_pointer = (reservation_start as usize)
            .checked_add(reservation_layout.thread_pointer_offset())
            .unwrap(); // TODO return error
        if cfg!(target_arch = "x86_64") {
            let thread_pointer_slice = &mut reservation
                [reservation_layout.thread_pointer_offset()..][..mem::size_of::<usize>()];
            thread_pointer_slice.copy_from_slice(&thread_pointer.to_ne_bytes());
        }
        thread_pointer
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn initialize_exact_reservation_region(
        &self,
        exact_reservation: &Region,
    ) -> Result<usize, RegionLayoutError> {
        if exact_reservation.fits_exactly(self.reservation_layout().footprint()) {
            Ok(self.initialize_reservation(exact_reservation.start()))
        } else {
            Err(RegionLayoutError::new())
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn initialize_inexact_reservation_region(
        &self,
        inexact_reservation: &Region,
    ) -> Result<usize, RegionLayoutError> {
        if let Ok(TrimmedRegion { trimmed, .. }) =
            inexact_reservation.trim(self.reservation_layout().footprint())
        {
            Ok(self.initialize_exact_reservation_region(&trimmed).unwrap())
        } else {
            Err(RegionLayoutError::new())
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegionLayoutError(());

impl RegionLayoutError {
    fn new() -> Self {
        Self(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TlsReservationLayout {
    footprint: Layout,
    segment_offset: usize,
    thread_pointer_offset: usize,
}

impl TlsReservationLayout {
    fn from_segment_layout(segment_layout: Layout) -> Self {
        if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
            let tcb_size = 2 * mem::size_of::<usize>();
            let tcb_layout = Layout::from_size_align(tcb_size, tcb_size).unwrap();
            let (footprint, segment_offset) = tcb_layout.extend(segment_layout).unwrap();
            Self {
                footprint,
                segment_offset,
                thread_pointer_offset: 0,
            }
        } else if cfg!(any(target_arch = "riscv32", target_arch = "riscv64")) {
            Self {
                footprint: segment_layout,
                segment_offset: 0,
                thread_pointer_offset: 0,
            }
        } else if cfg!(target_arch = "x86_64") {
            let tcb_layout =
                Layout::from_size_align(2 * mem::size_of::<usize>(), mem::size_of::<usize>())
                    .unwrap(); // could probably get away with just 1x word size for size (keeping 2x word size alignment)
            let (footprint, thread_pointer_offset) = segment_layout.extend(tcb_layout).unwrap();
            Self {
                footprint,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Region {
    start: *mut u8,
    size: usize,
}

impl Region {
    pub const fn new(start: *mut u8, size: usize) -> Self {
        Self { start, size }
    }

    pub const fn start(&self) -> *mut u8 {
        self.start
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    fn fits_exactly(&self, layout: Layout) -> bool {
        self.size() == layout.size() && self.start().align_offset(layout.align()) == 0
    }

    fn trim(&self, layout: Layout) -> Result<TrimmedRegion, TrimRegionError> {
        let start_addr = self.start() as usize;
        let trimmed_start_addr = start_addr
            .checked_next_multiple_of(layout.align())
            .ok_or(TrimRegionError::new())?;
        let remainder_start_addr = trimmed_start_addr
            .checked_add(layout.size())
            .ok_or(TrimRegionError::new())?;
        let remainder_end_addr = start_addr
            .checked_add(self.size())
            .ok_or(TrimRegionError::new())?;
        if remainder_start_addr > remainder_end_addr {
            return Err(TrimRegionError::new());
        }
        Ok(TrimmedRegion {
            padding: Region::new(start_addr as *mut u8, trimmed_start_addr - start_addr),
            trimmed: Region::new(
                trimmed_start_addr as *mut u8,
                remainder_start_addr - trimmed_start_addr,
            ),
            remainder: Region::new(
                remainder_start_addr as *mut u8,
                remainder_end_addr - remainder_start_addr,
            ),
        })
    }
}

struct TrimmedRegion {
    #[allow(dead_code)]
    padding: Region,
    trimmed: Region,
    #[allow(dead_code)]
    remainder: Region,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TrimRegionError(());

impl TrimRegionError {
    fn new() -> Self {
        Self(())
    }
}
