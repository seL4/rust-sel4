//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::path::Path;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Error, Parse, ParseStream};
use syn::{parse_macro_input, ItemMod, MetaNameValue, Result};

/// Set a module's `#[path]` relative to `$OUT_DIR`.
///
/// The following is not supported by rustc:
/// ```ignore
/// #[path = concat!(env!("OUT_DIR"), "/foo.rs")]
/// mod foo;
/// ```
///
/// This macro does exactly that:
/// ```ignore
/// #[in_out_dir]
/// mod foo;
/// ```
///
/// This works too:
/// ```ignore
/// #[in_out_dir(relative_path = "path/to/bar.rs")]
/// mod foo;
/// ```
#[proc_macro_attribute]
pub fn in_out_dir(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as Input);
    let item_mod = parse_macro_input!(item as ItemMod);
    let relative_path = input
        .relative_path
        .unwrap_or_else(|| format!("{}.rs", item_mod.ident));
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join(relative_path);
    let path = path.to_str().unwrap();
    quote!(
        #[path = #path]
        #item_mod
    )
    .into()
}

struct Input {
    relative_path: Option<String>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let relative_path = if input.is_empty() {
            None
        } else {
            let name_value = input.parse::<MetaNameValue>()?;
            let path = name_value.path;
            if !path.is_ident("relative_path") {
                return Err(Error::new_spanned(path, "unrecognized argument"));
            }
            let value = name_value.value;
            match value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => Some(s.value()),
                _ => return Err(Error::new_spanned(value, "value must be a string literal")),
            }
        };
        Ok(Self { relative_path })
    }
}
