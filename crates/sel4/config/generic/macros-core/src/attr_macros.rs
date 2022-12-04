use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse2, spanned::Spanned};

use sel4_config_generic_types::Configuration;

use crate::{parse_or_return, Evaluator};

macro_rules! ensure_empty {
    ($tokenstream:ident) => {
        if !$tokenstream.is_empty() {
            return quote_spanned! {
                $tokenstream.span() => compile_error!("expected empty");
            };
        }
    };
}

pub fn cfg_impl(config: &Configuration, input: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_or_return!(input as syn::NestedMeta);
    let r = Evaluator::new(config).eval_nested_meta(&attr);
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

pub fn cfg_struct_impl(
    inner_attr: &str,
    config: &Configuration,
    input: TokenStream,
    item: TokenStream,
) -> TokenStream {
    ensure_empty!(input);
    let mut struct_item = parse_or_return!(item as syn::ItemStruct);
    let mut helper = Helper::new(inner_attr, config);
    match &mut struct_item.fields {
        syn::Fields::Named(fields) => {
            fields.named = helper.filter_punctuated(fields.named.clone(), |field| &mut field.attrs);
        }
        syn::Fields::Unnamed(fields) => {
            fields.unnamed =
                helper.filter_punctuated(fields.unnamed.clone(), |field| &mut field.attrs);
        }
        syn::Fields::Unit => {}
    }
    helper.first_err_or(struct_item)
}

pub fn cfg_enum_impl(
    inner_attr: &str,
    config: &Configuration,
    input: TokenStream,
    item: TokenStream,
) -> TokenStream {
    ensure_empty!(input);
    let mut enum_item = parse_or_return!(item as syn::ItemEnum);
    let mut helper = Helper::new(inner_attr, config);
    enum_item.variants =
        helper.filter_punctuated(enum_item.variants.clone(), |variant| &mut variant.attrs);
    helper.first_err_or(enum_item)
}

pub fn cfg_match_impl(
    inner_attr: &str,
    config: &Configuration,
    input: TokenStream,
    item: TokenStream,
) -> TokenStream {
    ensure_empty!(input);
    let mut match_expr = parse_or_return!(item as syn::ExprMatch);
    let mut helper = Helper::new(inner_attr, config);
    match_expr
        .arms
        .drain_filter(|arm| !helper.process_attrs(&mut arm.attrs));
    helper.first_err_or(match_expr)
}

struct Helper<'a> {
    inner_attr: &'a str,
    evaluator: Evaluator<'a>,
    first_err: Option<TokenStream>,
}

impl<'a> Helper<'a> {
    fn new(inner_attr: &'a str, config: &'a Configuration) -> Self {
        Self {
            inner_attr,
            evaluator: Evaluator::new(config),
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
        let inner_attr = self.inner_attr.clone();
        let keep = attrs
            .drain_filter(|attr| attr.path.is_ident(&format_ident!("{}", &inner_attr)))
            .all(|attr| match attr.parse_args::<syn::NestedMeta>() {
                Ok(expr) => {
                    let r = self.evaluator.eval_nested_meta(&expr);
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
