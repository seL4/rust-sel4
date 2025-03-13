//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::marker::PhantomData;

use proc_macro::TokenStream;

use sel4_config_data::get_kernel_config;

mod generic;

use generic::MacroImpls;

fn get_impls() -> MacroImpls<'static> {
    MacroImpls::new(get_kernel_config(), "sel4_cfg")
}

/// Make the attached code conditional on a seL4 kernel configuration expression.
///
/// Supports the same syntax as `#[cfg]`, except primitive expressions are based on seL4 kernel
/// configuration rather than `rustc` configuration.
///
/// Suppose `$SEL4_PREFIX/libsel4/include/kernel/gen_config.json` contains:
///
/// ```ignore
/// {
///     "KERNEL_MCS": false,
///     "HAVE_FPU": true
///     "ARM_PLAT": "bcm2711",
///     "MAX_NUM_NODES": "4",
///     ...
/// }
/// ```
///
/// Note that values are either booleans or strings. Configuration keys corresponding to boolean
/// values are used as in `#[sel4_cfg(KERNEL_MCS)]`, whereas those corresponding to strings are used
/// in equalities as in `#[sel4_cfg(MAX_NUM_NODES = 4)]`. Expressions can be combined using `not()`,
/// `any()`, and `all()`, just like expressions in `#[cfg]`.
///
/// Unlike in `#[cfg]`, using a configuration key that is not present in the seL4 kernel
/// configuration will result in an error rather than just evaluating to false. That is, a key that
/// is not part of an equality expression evaluates to itsqqqqqqqqqq boolean value, whereas in `#[cfg]` a key
/// that is not part of an equality expression evaluates to whether it is present in the
/// configuration.
#[proc_macro_attribute]
pub fn sel4_cfg(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls().cfg_impl(input.into(), item.into()).into()
}

/// Make the associated attribute expression conditional on a seL4 kernel configuration expression.
///
/// Supports the same syntax as `#[cfg_attr]`, except primitive expressions are based on seL4 kernel
/// configuration rather than `rustc` configuration.
///
/// See [`macro@sel4_cfg`].
#[proc_macro_attribute]
pub fn sel4_cfg_attr(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls().cfg_attr_impl(input.into(), item.into()).into()
}

/// Allows a `struct`'s fields to use the [`macro@sel4_cfg`] attribute.
///
/// # Example
///
/// ```ignore
/// #[sel4_cfg_struct]
/// struct Foo {
///     #[sel4_cfg(KERNEL_MCS)]
///     bar: bool,
/// }
/// ```
#[proc_macro_attribute]
pub fn sel4_cfg_struct(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls()
        .cfg_struct_impl(input.into(), item.into())
        .into()
}

/// Allows an `enum`'s variants to use the [`macro@sel4_cfg`] attribute.
///
/// # Example
///
/// ```ignore
/// #[sel4_cfg_enum]
/// enum Foo {
///     #[sel4_cfg(KERNEL_MCS)]
///     Bar,
/// }
/// ```
#[proc_macro_attribute]
pub fn sel4_cfg_enum(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls().cfg_enum_impl(input.into(), item.into()).into()
}

/// Allows a `match` expression's variants to use the [`macro@sel4_cfg`] attribute.
///
/// Using this attribute macro currently requires nightly (`#![feature(proc_macro_hygiene)]` and
/// `#![feature(stmt_expr_attributes)]`). The less elegant [`macro@sel4_cfg_wrap_match`] serves the
/// same purpose and works on stable.
///
/// # Example
///
/// ```ignore
/// #[sel4_cfg_match]
/// match foo {
///     #[sel4_cfg(KERNEL_MCS)]
///     1337 => bar,
/// }
/// ```
#[proc_macro_attribute]
pub fn sel4_cfg_match(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls().cfg_match_impl(input.into(), item.into()).into()
}

/// Like [`macro@sel4_cfg_match`], except it works on stable, at the expense of not being an
/// attribute macro.
///
/// # Example
///
/// ```ignore
/// sel4_cfg_wrap_match! {
///     match foo {
///         #[sel4_cfg(KERNEL_MCS)]
///         1337 => bar,
///     }
/// }
/// ```
#[proc_macro]
pub fn sel4_cfg_wrap_match(item: TokenStream) -> TokenStream {
    get_impls().cfg_wrap_match_impl(item.into()).into()
}

/// Like `cfg_if::cfg_if!`, except with [`macro@sel4_cfg`] instead of `#[cfg]`.
///
/// # Example
///
/// ```ignore
/// sel4_cfg_if! {
///     if #[sel4_cfg(KERNEL_MCS)] {
///         ...
///     } else if #[sel4_cfg(MAX_NUM_NODES = "1")] {
///         ...
///     } else {
///         ...
///     }
/// }
/// ```
#[proc_macro]
pub fn sel4_cfg_if(toks: TokenStream) -> TokenStream {
    get_impls().cfg_if_impl(toks.into()).into()
}

/// Like `core::cfg!`, except using the seL4 kernel configuration.
///
/// Unlike `core::cfg!`, this macro requires the configuration key to correspond to a boolean
/// value.
///
/// See [`macro@sel4_cfg`] for documentation on the configuration expression syntax.
///
/// # Example
///
/// ```ignore
/// if sel4_cfg_bool!(KERNEL_MCS) {
///     ...
/// }
/// ```
#[proc_macro]
pub fn sel4_cfg_bool(key_toks: TokenStream) -> TokenStream {
    get_impls().cfg_bool_impl(key_toks.into()).into()
}

/// Like `core::cfg!`, except using the seL4 kernel configuration.
///
/// This macro requires the configuration key to correspond to a string value. It parses that value
/// into an integer at compile-time, and assignes to it the type `usize`.
///
/// See [`macro@sel4_cfg`] for documentation on the configuration expression syntax.
///
/// # Example
///
/// ```ignore
/// assert_eq!(1usize, sel4_cfg_usize!(MAX_NUM_NODES));
/// ```
#[proc_macro]
pub fn sel4_cfg_str(key_toks: TokenStream) -> TokenStream {
    get_impls()
        .cfg_from_str_impl::<String>(PhantomData, key_toks.into())
        .into()
}

/// Like `core::cfg!`, except using the seL4 kernel configuration.
///
/// This macro requires the configuration key to correspond to a string value. It parses that value
/// into an integer at compile-time, and assigns it type `usize`.
///
/// See [`macro@sel4_cfg`] for documentation on the configuration expression syntax.
///
/// # Example
///
/// ```ignore
/// assert_eq!(1usize, sel4_cfg_usize!(MAX_NUM_NODES));
/// ```
#[proc_macro]
pub fn sel4_cfg_usize(key_toks: TokenStream) -> TokenStream {
    get_impls()
        .cfg_from_str_impl::<usize>(PhantomData, key_toks.into())
        .into()
}

/// Like `core::cfg!`, except using the seL4 kernel configuration.
///
/// This macro requires the configuration key to correspond to a string value. It parses that value
/// into an integer at compile-time, and assigns to it the one of the types `u32` or `u64`,
/// depending on the value of `WORD_SIZE` configuration key.
///
/// See [`macro@sel4_cfg`] for documentation on the configuration expression syntax.
///
/// # Example
///
/// ```ignore
/// assert_eq!(1u64, sel4_cfg_word!(MAX_NUM_NODES));
/// ```
#[proc_macro]
pub fn sel4_cfg_word(key_toks: TokenStream) -> TokenStream {
    let impls = get_impls();
    let word_size = impls
        .config()
        .get("WORD_SIZE")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let toks = match word_size {
        32 => impls.cfg_from_str_impl::<u32>(PhantomData, key_toks.into()),
        64 => impls.cfg_from_str_impl::<u64>(PhantomData, key_toks.into()),
        _ => panic!(),
    };
    toks.into()
}
