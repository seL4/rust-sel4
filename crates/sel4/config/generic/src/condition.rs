//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use fallible_iterator::FallibleIterator;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::parse::{Parse, ParseStream, Parser, Result as ParseResult};
use syn::spanned::Spanned;
use syn::Token;

use sel4_config_generic_types::{Configuration, Value};

pub(crate) enum Condition {
    Key(syn::Ident),
    KeyValue(syn::Ident, syn::LitStr),
    Not(Box<Condition>),
    All(Vec<Condition>),
    Any(Vec<Condition>),
}

impl Condition {
    pub(crate) fn eval(&self, config: &Configuration) -> Result<bool, EvalError> {
        ConfigurationForEval(config).eval(self)
    }
}

impl Parse for Condition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(match input.parse()? {
            syn::Meta::Path(path) => Self::Key(path.require_ident()?.clone()),
            syn::Meta::NameValue(meta_name_value) => {
                let key = meta_name_value.path.require_ident()?;
                let value = match meta_name_value.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) => s,
                    v => return Err(syn::Error::new_spanned(v, "value must be a string literal")),
                };
                Self::KeyValue(key.clone(), value)
            }
            syn::Meta::List(meta_list) => {
                match meta_list.path.require_ident()?.to_string().as_str() {
                    "not" => Self::Not(meta_list.parse_args()?),
                    "all" => Self::All(
                        (|input_: ParseStream| {
                            input_.parse_terminated(Condition::parse, Token![,])
                        })
                        .parse2(meta_list.tokens)?
                        .into_iter()
                        .collect(),
                    ),
                    "any" => Self::Any(
                        (|input_: ParseStream| {
                            input_.parse_terminated(Condition::parse, Token![,])
                        })
                        .parse2(meta_list.tokens)?
                        .into_iter()
                        .collect(),
                    ),
                    _ => return Err(input.error("unexpected operation")),
                }
            }
        })
    }
}

pub(crate) struct EvalError {
    pub(crate) span: Span,
    pub(crate) message: String,
}

impl EvalError {
    pub(crate) fn new(span: Span, message: String) -> Self {
        Self { span, message }
    }

    pub(crate) fn render(&self) -> TokenStream {
        let message = &self.message;
        quote_spanned! {
            self.span => compile_error!(#message);
        }
    }
}

fn err<T, U: ToString>(node: impl Spanned, message: U) -> Result<T, EvalError> {
    Err(EvalError::new(node.span(), message.to_string()))
}

struct ConfigurationForEval<'a>(&'a Configuration);

impl<'a> ConfigurationForEval<'a> {
    fn lookup_key(&self, k: &syn::Ident) -> Result<&Value, EvalError> {
        self.0
            .get(&k.to_string())
            .ok_or_else(|| EvalError::new(k.span(), format!("unknown config key '{k}'")))
    }

    fn eval(&self, cond: &Condition) -> Result<bool, EvalError> {
        Ok(match cond {
            Condition::Key(k) => match self.lookup_key(k)? {
                Value::Bool(v) => *v,
                _ => return err(k, "config key does not correspond to a boolean"),
            },
            Condition::KeyValue(k, v) => match self.lookup_key(k)? {
                Value::String(v_) => v_ == &v.value(),
                _ => return err(k, "config key does not correspond to a string"),
            },
            Condition::Not(cond_) => !self.eval(cond_)?,
            Condition::All(conds) => {
                fallible_iterator::convert(conds.iter().map(Ok)).all(|cond_| self.eval(cond_))?
            }
            Condition::Any(conds) => {
                fallible_iterator::convert(conds.iter().map(Ok)).any(|cond_| self.eval(cond_))?
            }
        })
    }
}
