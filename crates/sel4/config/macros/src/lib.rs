use std::marker::PhantomData;

use proc_macro::TokenStream;

use sel4_config_data::get_kernel_config;
use sel4_config_generic_macros_core::Impls;

fn get_impls() -> Impls<'static> {
    Impls::new(get_kernel_config(), "sel4_cfg")
}

#[proc_macro_attribute]
pub fn sel4_cfg(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls().cfg_impl(input.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn sel4_cfg_attr(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls().cfg_attr_impl(input.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn sel4_cfg_struct(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls()
        .cfg_struct_impl(input.into(), item.into())
        .into()
}

#[proc_macro_attribute]
pub fn sel4_cfg_enum(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls().cfg_enum_impl(input.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn sel4_cfg_match(input: TokenStream, item: TokenStream) -> TokenStream {
    get_impls().cfg_match_impl(input.into(), item.into()).into()
}

#[proc_macro]
pub fn sel4_cfg_bool(key_toks: TokenStream) -> TokenStream {
    get_impls().cfg_bool_impl(key_toks.into()).into()
}

#[proc_macro]
pub fn sel4_cfg_str(key_toks: TokenStream) -> TokenStream {
    get_impls()
        .cfg_from_str_impl::<String>(PhantomData, key_toks.into())
        .into()
}

#[proc_macro]
pub fn sel4_cfg_usize(key_toks: TokenStream) -> TokenStream {
    get_impls()
        .cfg_from_str_impl::<usize>(PhantomData, key_toks.into())
        .into()
}

#[proc_macro]
pub fn sel4_cfg_if(toks: TokenStream) -> TokenStream {
    get_impls().cfg_if_impl(toks.into()).into()
}
