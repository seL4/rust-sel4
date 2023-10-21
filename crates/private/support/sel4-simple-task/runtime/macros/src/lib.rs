//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    main_impl(quote!(run_main), item)
}

#[proc_macro_attribute]
pub fn main_json(_attr: TokenStream, item: TokenStream) -> TokenStream {
    main_impl(quote!(run_main_json), item)
}

#[proc_macro_attribute]
pub fn main_postcard(_attr: TokenStream, item: TokenStream) -> TokenStream {
    main_impl(quote!(run_main_postcard), item)
}

fn main_impl(f: TokenStream2, item: TokenStream) -> TokenStream {
    let macro_path = quote!(sel4_simple_task_runtime::declare_main_with);
    let item = parse_macro_input!(item as ItemFn);
    let ident = &item.sig.ident;
    (quote! {
        #macro_path!(#f, #ident);

        #item
    })
    .into()
}
