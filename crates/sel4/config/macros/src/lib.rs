extern crate proc_macro;

use std::marker::PhantomData;

use proc_macro::TokenStream;

use sel4_config_data::get_kernel_config as get_config;
use sel4_config_generic_macros_core::{
    cfg_attr_impl, cfg_bool_impl, cfg_enum_impl, cfg_from_str_impl, cfg_if_impl, cfg_impl,
    cfg_match_impl, cfg_struct_impl,
};

const SYNTHETIC_ATTRIBUTE: &str = "sel4_cfg";

#[proc_macro_attribute]
pub fn sel4_cfg(input: TokenStream, item: TokenStream) -> TokenStream {
    cfg_impl(get_config(), input.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn sel4_cfg_attr(input: TokenStream, item: TokenStream) -> TokenStream {
    cfg_attr_impl(get_config(), input.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn sel4_cfg_struct(input: TokenStream, item: TokenStream) -> TokenStream {
    cfg_struct_impl(SYNTHETIC_ATTRIBUTE, get_config(), input.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn sel4_cfg_enum(input: TokenStream, item: TokenStream) -> TokenStream {
    cfg_enum_impl(SYNTHETIC_ATTRIBUTE, get_config(), input.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn sel4_cfg_match(input: TokenStream, item: TokenStream) -> TokenStream {
    cfg_match_impl(SYNTHETIC_ATTRIBUTE, get_config(), input.into(), item.into()).into()
}

#[proc_macro]
pub fn sel4_cfg_bool(key_toks: TokenStream) -> TokenStream {
    cfg_bool_impl(get_config(), key_toks.into()).into()
}

#[proc_macro]
pub fn sel4_cfg_str(key_toks: TokenStream) -> TokenStream {
    cfg_from_str_impl::<String>(PhantomData, get_config(), key_toks.into()).into()
}

#[proc_macro]
pub fn sel4_cfg_usize(key_toks: TokenStream) -> TokenStream {
    cfg_from_str_impl::<usize>(PhantomData, get_config(), key_toks.into()).into()
}

#[proc_macro]
pub fn sel4_cfg_if(toks: TokenStream) -> TokenStream {
    cfg_if_impl(get_config(), toks.into()).into()
}
