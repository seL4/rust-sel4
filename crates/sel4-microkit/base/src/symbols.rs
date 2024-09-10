//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;
use core::str::{self, Utf8Error};

#[macro_export]
macro_rules! var {
    ($(#[$attrs:meta])* $symbol:ident: $ty:ty = $default:expr) => {{
        use $crate::_private::ImmutableCell;

        $(#[$attrs])*
        #[no_mangle]
        #[link_section = ".data"]
        static $symbol: ImmutableCell<$ty> = ImmutableCell::new($default);

        $symbol.get()
    }}
}

/// Declares a symbol via which the `microkit` tool can inject a memory region's address, and
/// returns the memory region's address at runtime.
///
/// For more detail, see its definition.
///
/// The patching mechanism used by the `microkit` tool requires that the symbol be allocated space
/// in the protection domain's ELF file, so we delare the symbol as part of the `.data` section.
///
/// # Examples
///
/// ```rust
/// let region_1 = unsafe {
///     ExternallySharedRef::<'static, Foo>::new(
///         memory_region_symbol!(region_1_addr: *mut Foo),
///     )
/// };
///
/// let region_2 = unsafe {
///     ExternallySharedRef::<'static, [u8]>::new_read_only(
///         memory_region_symbol!(region_2_addr: *mut [u8], n = REGION_2_SIZE),
///     )
/// };
/// ```
///
/// # Note
///
/// The `microkit` tool requires memory region address symbols to be present in protection domain
/// binaries. To prevent Rust from optimizing them out in cases where it is not used, add the
/// unstable `#[used(linker)]` attribute. For example:
///
/// ```rust
/// #![feature(used_with_arg)]
///
/// // might be optimized away if not used
/// memory_region_symbol!(region_addr: *mut Foo)
///
/// // won't be optimized away
/// memory_region_symbol! {
///     #[used(linker)]
///     region_addr: *mut Foo
/// }
/// ```
#[macro_export]
macro_rules! memory_region_symbol {
    ($(#[$attrs:meta])* $symbol:ident: *mut [$ty:ty], n = $n:expr, bytes = $bytes:expr $(,)?) => {
        core::ptr::NonNull::slice_from_raw_parts(
            $crate::memory_region_symbol!(
                $(#[$attrs])* $symbol: *mut [$ty; $n], bytes = $bytes
            ).cast::<$ty>(),
            $n,
        )
    };
    ($(#[$attrs:meta])* $symbol:ident: *mut [$ty:ty], n = $n:expr $(,)?) => {
        core::ptr::NonNull::slice_from_raw_parts(
            $crate::memory_region_symbol!(
                $(#[$attrs])* $symbol: *mut [$ty; $n]
            ).cast::<$ty>(),
            $n,
        )
    };
    ($(#[$attrs:meta])* $symbol:ident: *mut $ty:ty, bytes = $bytes:expr $(,)?) => {{
        const _: () = assert!($bytes == core::mem::size_of::<$ty>());
        $crate::memory_region_symbol!($(#[$attrs])* $symbol: *mut $ty)
    }};
    ($(#[$attrs:meta])* $symbol:ident: *mut $ty:ty $(,)?) => {
        core::ptr::NonNull::new(
            *$crate::var!($(#[$attrs])* $symbol: usize = 0) as *mut $ty
        ).unwrap_or_else(|| {
            panic!("{} is null", stringify!($symbol))
        })
    };
}

#[cfg(not(feature = "extern-symbols"))]
macro_rules! maybe_extern_var {
    ($symbol:ident: $ty:ty = $default:expr) => {
        var! {
            #[used(linker)]
            $symbol: $ty = $default
        }
    };
}

#[cfg(feature = "extern-symbols")]
macro_rules! maybe_extern_var {
    ($symbol:ident: $ty:ty = $default:expr) => {{
        extern "C" {
            static $symbol: $ty;
        }

        unsafe { &$symbol }
    }};
}

/// Returns whether this projection domain is a passive server.
pub fn pd_is_passive() -> bool {
    *maybe_extern_var!(microkit_passive: bool = false)
}

/// Returns the name of this projection domain without converting to unicode.
pub fn pd_name_bytes() -> &'static [u8] {
    let all_bytes = maybe_extern_var!(microkit_name: [u8; 16] = [0; 16]);
    let n = all_bytes.iter().take_while(|b| **b != 0).count();
    &all_bytes[..n]
}

/// Returns the name of this projection domain.
pub fn pd_name() -> Result<&'static str, Utf8Error> {
    str::from_utf8(pd_name_bytes())
}

pub fn ipc_buffer_ptr() -> *mut sel4::IpcBuffer {
    extern "C" {
        static mut __sel4_ipc_buffer_obj: sel4::IpcBuffer;
    }

    ptr::addr_of_mut!(__sel4_ipc_buffer_obj)
}
