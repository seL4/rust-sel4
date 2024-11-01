//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::alloc::Layout;

use crate::{Region, TlsImage};

pub struct HeapTlsReservation {
    start: *mut u8,
    layout: Layout,
    thread_pointer: usize,
}

impl HeapTlsReservation {
    fn initialize(tls_image: &TlsImage) -> Self {
        let layout = tls_image.reservation_layout().footprint();
        let start = unsafe { ::alloc::alloc::alloc(layout) };
        let region = Region::new(start, layout.size());
        let thread_pointer =
            unsafe { tls_image.initialize_exact_reservation_region(&region) }.unwrap();
        Self {
            start,
            layout,
            thread_pointer,
        }
    }

    pub fn thread_pointer(&self) -> usize {
        self.thread_pointer
    }
}

impl Drop for HeapTlsReservation {
    fn drop(&mut self) {
        unsafe {
            ::alloc::alloc::dealloc(self.start, self.layout);
        }
    }
}

impl TlsImage {
    pub fn initialize_on_heap(&self) -> HeapTlsReservation {
        HeapTlsReservation::initialize(self)
    }
}
