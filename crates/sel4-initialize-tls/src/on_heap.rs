//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::{TlsImage, TlsReservationLayout};

pub struct HeapTlsReservation {
    start: *mut u8,
    layout: TlsReservationLayout,
}

impl HeapTlsReservation {
    fn new(tls_image: &TlsImage) -> Self {
        let layout = tls_image.reservation_layout();
        let start = unsafe { ::alloc::alloc::alloc(layout.footprint()) };
        unsafe {
            tls_image.initialize_tls_reservation(start);
        };
        Self { start, layout }
    }

    pub fn thread_pointer(&self) -> usize {
        (self.start as usize) + self.layout.thread_pointer_offset()
    }
}

impl Drop for HeapTlsReservation {
    fn drop(&mut self) {
        unsafe {
            ::alloc::alloc::dealloc(self.start, self.layout.footprint());
        }
    }
}

impl TlsImage {
    pub fn initialize_on_heap(&self) -> HeapTlsReservation {
        HeapTlsReservation::new(self)
    }
}
