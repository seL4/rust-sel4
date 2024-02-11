//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::any;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse2, spanned::Spanned};

use sel4_config_generic_types::{Key, Value};

use crate::{parse_or_return, MacroImpls};

impl<'a> MacroImpls<'a> {
    pub fn cfg_generic_impl<T>(
        &self,
        key_toks: TokenStream,
        f: impl FnOnce((&Key, &Value)) -> Result<T, String>,
    ) -> TokenStream
    where
        T: ToTokens,
    {
        let ident = parse_or_return!(key_toks as syn::Ident);
        let key = format!("{ident}");
        match self.config().get(&key) {
            Some(value) => f((&key, value)).map(|to_tokens| quote!(#to_tokens)),
            None => Err(format!("unknown config key '{key}'")),
        }
        .unwrap_or_else(|message| {
            quote_spanned! {
                key.span() => compile_error!(#message);
            }
        })
    }

    pub fn cfg_bool_impl(&self, key_toks: TokenStream) -> TokenStream {
        self.cfg_generic_impl(key_toks, |(key, value)| match value {
            Value::Bool(value) => Ok(*value),
            _ => Err(format!(
                "value corresponding to config key '{key}' is not boolean"
            )),
        })
    }

    pub fn cfg_from_str_impl<T>(
        &self,
        _phantom: PhantomData<T>,
        key_toks: TokenStream,
    ) -> TokenStream
    where
        T: FromStr + ToTokens,
        <T as FromStr>::Err: Debug,
    {
        self.cfg_generic_impl(key_toks, |(key, value)| match value {
            Value::String(value) => value.parse::<T>().map_err(|err| {
                format!(
                    "error parsing value corresponding to config key '{}' as a '{}': {:?}",
                    key,
                    any::type_name::<T>(),
                    err,
                )
            }),
            _ => Err(format!(
                "value corresponding to config key '{key}' is not string"
            )),
        })
    }
}
