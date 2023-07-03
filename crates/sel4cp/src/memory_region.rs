//! Utilities for declaring and using share memory regions.

pub use zerocopy::{AsBytes, FromBytes};

pub use sel4_externally_shared::access::{ReadOnly, ReadWrite};
pub use sel4_externally_shared::{ExternallySharedPtr, ExternallySharedRef};

/// Declares a symbol via which the `sel4cp` tool can inject a memory region's address, and returns
/// the memory region's address at runtime.
///
/// This is its definition:
///
/// ```rust
/// #[macro_export]
/// macro_rules! memory_region_symbol {
///     ($symbol:ident: *mut [$ty:ty], n = $n:expr) => {{
///         core::ptr::NonNull::slice_from_raw_parts(
///             $crate::memory_region_symbol!($symbol: *mut $ty),
///             $n,
///         )
///     }};
///     ($symbol:ident: *mut $ty:ty) => {{
///         #[no_mangle]
///         #[link_section = ".data"]
///         static mut $symbol: *mut $ty = core::ptr::null_mut();
///
///         core::ptr::NonNull::new($symbol).unwrap_or_else(|| {
///             panic!("{} is null", stringify!($symbol))
///         })
///     }};
/// }
/// ```
///
/// The patching mechanism used by the `sel4cp` tool requires that the symbol be allocated space in
/// the protection domain's ELF file, so we delare the symbol as part of the `.data` section.
#[macro_export]
macro_rules! memory_region_symbol {
    ($symbol:ident: *mut [$ty:ty], n = $n:expr) => {{
        core::ptr::NonNull::slice_from_raw_parts(
            $crate::memory_region_symbol!($symbol: *mut $ty),
            $n,
        )
    }};
    ($symbol:ident: *mut $ty:ty) => {{
        #[no_mangle]
        #[link_section = ".data"]
        static mut $symbol: *mut $ty = core::ptr::null_mut();

        core::ptr::NonNull::new($symbol).unwrap_or_else(|| {
            panic!("{} is null", stringify!($symbol))
        })
    }};
}

pub use memory_region_symbol;
