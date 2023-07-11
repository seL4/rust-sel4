//! Utilities for declaring and using share memory regions.

use core::mem;
use core::ptr::NonNull;

/// Declares a symbol via which the `sel4cp` tool can inject a memory region's address, and returns
/// the memory region's address at runtime.
///
/// This is its definition:
///
/// ```rust
/// #[macro_export]
/// macro_rules! memory_region_symbol {
///     ($(#[$attrs:meta])* $symbol:ident: *mut [$ty:ty], n = $n:expr) => {{
///         core::ptr::NonNull::slice_from_raw_parts(
///             $crate::memory_region_symbol!($(#[$attrs])* $symbol: *mut $ty),
///             $n,
///         )
///     }};
///     ($(#[$attrs:meta])* $symbol:ident: *mut $ty:ty) => {{
///         $(#[$attrs])*
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
/// The `sel4cp` tool requires memory region address symbols to be present in protection domain
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
    ($(#[$attrs:meta])* $symbol:ident: *mut [$ty:ty], n = $n:expr) => {{
        core::ptr::NonNull::slice_from_raw_parts(
            $crate::memory_region_symbol!($(#[$attrs])* $symbol: *mut $ty),
            $n,
        )
    }};
    ($(#[$attrs:meta])* $symbol:ident: *mut $ty:ty) => {{
        $(#[$attrs])*
        #[no_mangle]
        #[link_section = ".data"]
        static mut $symbol: *mut $ty = core::ptr::null_mut();

        core::ptr::NonNull::new($symbol).unwrap_or_else(|| {
            panic!("{} is null", stringify!($symbol))
        })
    }};
}

pub use memory_region_symbol;

pub fn cast_memory_region_checked<T: Sized>(bytes_ptr: NonNull<[u8]>) -> NonNull<T> {
    let ptr = bytes_ptr.cast::<T>();
    assert!(ptr.as_ptr().is_aligned());
    assert!(mem::size_of::<T>() <= bytes_ptr.len());
    ptr
}

pub fn cast_memory_region_to_slice_checked<T: Sized>(bytes_ptr: NonNull<[u8]>) -> NonNull<[T]> {
    let ptr = bytes_ptr.cast::<T>();
    assert!(ptr.as_ptr().is_aligned());
    assert!(bytes_ptr.len() % mem::size_of::<T>() == 0);
    let n = bytes_ptr.len() / mem::size_of::<T>();
    NonNull::slice_from_raw_parts(ptr, n)
}
