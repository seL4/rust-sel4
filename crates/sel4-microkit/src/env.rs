//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::str;

use sel4_immutable_cell::ImmutableCell;

use crate::abort;

extern "C" {
    static mut __sel4_ipc_buffer_obj: sel4::sys::seL4_IPCBuffer;
}

pub(crate) unsafe fn get_ipc_buffer() -> sel4::IPCBuffer {
    sel4::IPCBuffer::from_ptr(&mut __sel4_ipc_buffer_obj)
}

#[no_mangle]
#[used(linker)]
#[link_section = ".data"]
static passive: ImmutableCell<bool> = ImmutableCell::new(false); // just a placeholder

/// Returns whether this projection domain is a passive server.
pub fn pd_is_passive() -> bool {
    *passive.get()
}

#[no_mangle]
#[used(linker)]
#[link_section = ".data"]
static microkit_name: ImmutableCell<[u8; 16]> = ImmutableCell::new([0; 16]);

/// Returns the name of this projection domain.
pub fn pd_name() -> &'static str {
    let all_bytes = microkit_name.get();
    let bytes = match core::ffi::CStr::from_bytes_until_nul(all_bytes) {
        Ok(cstr) => cstr.to_bytes(),
        Err(_) => all_bytes,
    };
    str::from_utf8(bytes).unwrap_or_else(|_| {
        // abort to avoid recursive panic
        abort!("invalid embedded protection domain name");
    })
}

#[macro_export]
macro_rules! var {
    ($(#[$attrs:meta])* $symbol:ident: $ty:ty = $default:expr) => {{
        $(#[$attrs])*
        #[no_mangle]
        #[link_section = ".data"]
        static $symbol: $crate::_private::ImmutableCell<$ty> = $crate::_private::ImmutableCell::new($default);

        $symbol.get()
    }};
}
