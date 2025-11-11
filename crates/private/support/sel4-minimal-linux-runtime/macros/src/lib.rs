//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    let ident = &item.sig.ident;
    let attr = TokenStream2::from(attr);
    quote! {
        ::sel4_minimal_linux_runtime::declare_main!(main = #ident, #attr);

        #item
    }
    .into()
}
