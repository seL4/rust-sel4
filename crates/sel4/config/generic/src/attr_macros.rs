//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse2, spanned::Spanned, Token};

use crate::{parse_or_return, Condition, MacroImpls};

macro_rules! ensure_empty {
    ($tokenstream:ident) => {
        if !$tokenstream.is_empty() {
            return quote_spanned! {
                $tokenstream.span() => compile_error!("expected empty");
            };
        }
    };
}

impl<'a> MacroImpls<'a> {
    pub fn cfg_impl(&self, input: TokenStream, item: TokenStream) -> TokenStream {
        let cond = parse_or_return!(input as Condition);
        let r = self.eval(&cond);
        match r {
            Ok(pass) => {
                if pass {
                    item
                } else {
                    quote!()
                }
            }
            Err(err) => err.render(),
        }
    }

    pub fn cfg_attr_impl(&self, input: TokenStream, item: TokenStream) -> TokenStream {
        let input = parse_or_return!(input as CfgAttrInput);
        let r = self.eval(&input.condition);
        match r {
            Ok(pass) => {
                let this = if pass {
                    let body = &input.body;
                    quote!(#[#body])
                } else {
                    quote!()
                };
                quote! {
                    #this
                    #item
                }
            }
            Err(err) => err.render(),
        }
    }

    pub fn cfg_struct_impl(&self, input: TokenStream, item: TokenStream) -> TokenStream {
        ensure_empty!(input);
        let mut struct_item = parse_or_return!(item as syn::ItemStruct);
        let mut helper = Helper::new(self);
        match &mut struct_item.fields {
            syn::Fields::Named(fields) => {
                fields.named =
                    helper.filter_punctuated(fields.named.clone(), |field| &mut field.attrs);
            }
            syn::Fields::Unnamed(fields) => {
                fields.unnamed =
                    helper.filter_punctuated(fields.unnamed.clone(), |field| &mut field.attrs);
            }
            syn::Fields::Unit => {}
        }
        helper.first_err_or(struct_item)
    }

    pub fn cfg_enum_impl(&self, input: TokenStream, item: TokenStream) -> TokenStream {
        ensure_empty!(input);
        let mut enum_item = parse_or_return!(item as syn::ItemEnum);
        let mut helper = Helper::new(self);
        enum_item.variants =
            helper.filter_punctuated(enum_item.variants.clone(), |variant| &mut variant.attrs);
        helper.first_err_or(enum_item)
    }

    pub fn cfg_match_impl(&self, input: TokenStream, item: TokenStream) -> TokenStream {
        ensure_empty!(input);
        self.cfg_match_impl_inner(item)
    }

    pub fn cfg_wrap_match_impl(&self, item: TokenStream) -> TokenStream {
        self.cfg_match_impl_inner(item)
    }

    fn cfg_match_impl_inner(&self, item: TokenStream) -> TokenStream {
        let mut match_expr = parse_or_return!(item as syn::ExprMatch);
        let mut helper = Helper::new(self);
        match_expr
            .arms
            .retain_mut(|arm| helper.process_attrs(&mut arm.attrs));
        helper.first_err_or(match_expr)
    }
}

struct CfgAttrInput {
    condition: Condition,
    body: TokenStream,
}

impl syn::parse::Parse for CfgAttrInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let condition = input.parse()?;
        input.parse::<Token![,]>()?;
        let body = input.parse()?;
        Ok(Self { condition, body })
    }
}

struct Helper<'a> {
    impls: &'a MacroImpls<'a>,
    first_err: Option<TokenStream>,
}

impl<'a> Helper<'a> {
    fn new(impls: &'a MacroImpls<'a>) -> Self {
        Self {
            impls,
            first_err: None,
        }
    }

    fn report_err(&mut self, err: TokenStream) {
        if self.first_err.is_none() {
            self.first_err = Some(err)
        }
    }

    fn first_err_or(self, otherwise: impl ToTokens) -> TokenStream {
        match self.first_err {
            Some(err) => err,
            None => quote!(#otherwise),
        }
    }

    fn process_attrs(&mut self, attrs: &mut Vec<syn::Attribute>) -> bool /* keep */ {
        let synthetic_attr = self.impls.synthetic_attr();
        let key =
            |attr: &syn::Attribute| !attr.path().is_ident(&format_ident!("{}", synthetic_attr));
        attrs.sort_by_key(key);
        let keep = attrs.drain(attrs.partition_point(key)..).all(|attr| {
            match attr.parse_args::<Condition>() {
                Ok(cond) => {
                    let r = self.impls.eval(&cond);
                    match r {
                        Ok(pass) => pass,
                        Err(err) => {
                            self.report_err(err.render());
                            false
                        }
                    }
                }
                Err(err) => {
                    self.report_err(err.to_compile_error());
                    false
                }
            }
        });
        keep
    }

    fn filter_punctuated<T, P>(
        &mut self,
        punctuated: syn::punctuated::Punctuated<T, P>,
        f: impl Fn(&mut T) -> &mut Vec<syn::Attribute>,
    ) -> syn::punctuated::Punctuated<T, P> {
        filter_punctuated(punctuated, |value| self.process_attrs(f(value)))
    }
}

fn filter_punctuated<T, P>(
    punctuated: syn::punctuated::Punctuated<T, P>,
    mut f: impl FnMut(&mut T) -> bool,
) -> syn::punctuated::Punctuated<T, P> {
    let mut new = syn::punctuated::Punctuated::new();
    for pair in punctuated.into_pairs() {
        let (mut value, punct) = match pair {
            syn::punctuated::Pair::Punctuated(value, punct) => (value, Some(punct)),
            syn::punctuated::Pair::End(value) => (value, None),
        };
        if f(&mut value) {
            new.push_value(value);
            if let Some(punct) = punct {
                new.push_punct(punct);
            }
        }
    }
    new
}
