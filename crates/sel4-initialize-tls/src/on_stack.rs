//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_alloca::with_alloca_ptr;

use crate::{SetThreadPointerFn, TlsImage};

impl TlsImage {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn with_initialize_on_stack(
        &self,
        set_thread_pointer_fn: SetThreadPointerFn,
        f: impl FnOnce() -> !,
    ) -> ! {
        with_alloca_ptr(
            self.reservation_layout().footprint(),
            |tls_reservation_start| {
                self.initialize_reservation(tls_reservation_start);
                let thread_pointer = tls_reservation_start
                    .wrapping_byte_add(self.reservation_layout().thread_pointer_offset());
                (set_thread_pointer_fn)(thread_pointer as usize);
                f()
            },
        )
    }
}
