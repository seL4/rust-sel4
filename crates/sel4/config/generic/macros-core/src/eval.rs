use fallible_iterator::FallibleIterator;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;

use sel4_config_generic_types::{Configuration, Value};

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

pub(crate) struct Evaluator<'a> {
    config: &'a Configuration,
}

fn err<T, U: ToString>(node: impl Spanned, message: U) -> Result<T, EvalError> {
    Err(EvalError::new(node.span(), message.to_string()))
}

impl<'a> Evaluator<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    pub(crate) fn eval_nested_meta(&self, node: &syn::NestedMeta) -> Result<bool, EvalError> {
        Ok(match node {
            syn::NestedMeta::Meta(node) => self.eval_meta(node)?,
            syn::NestedMeta::Lit(node) => match node {
                syn::Lit::Bool(node) => node.value,
                _ => return err(node, "unexpected literal type"),
            },
        })
    }

    fn eval_meta(&self, node: &syn::Meta) -> Result<bool, EvalError> {
        Ok(match node {
            syn::Meta::Path(node) => {
                match self.lookup_path(node)? {
                    Value::Bool(v) => *v,
                    _ => return err(node, "config key does not correspond to a boolean"),
                }
            }
            syn::Meta::NameValue(node) => {
                match (&node.lit, self.lookup_path(&node.path)?) {
                    (syn::Lit::Str(l), Value::String(v)) => &l.value() == v,
                    (syn::Lit::Bool(l), Value::Bool(v)) => &l.value() == v,
                    _ => return err(node, "the type of the value corresponding to config key does not match the type of the value to which it is being compared"),
                }
            }
            syn::Meta::List(node) => {
                match node.path.get_ident() {
                    None => return err(&node.path, "unknown operation"),
                    Some(ident) => match ident.to_string().as_str() {
                        "not" => {
                            if node.nested.len() != 1 {
                                return err(&node.nested, "expected 1 argument")
                            }
                            !self.eval_nested_meta(node.nested.first().unwrap())?
                        }
                        "any" => {
                            fallible_iterator::convert(node.nested.iter().map(Ok)).any(|e| {
                                self.eval_nested_meta(e)
                            })?
                        }
                        "all" => {
                            fallible_iterator::convert(node.nested.iter().map(Ok)).all(|e| {
                                self.eval_nested_meta(e)
                            })?
                        }
                        _ => {
                            return err(&node.path, "unknown operation")
                        }
                    }
                }
            }
        })
    }

    fn lookup_path(&self, node: &syn::Path) -> Result<&Value, EvalError> {
        Ok(match node.get_ident() {
            None => return err(node, "not an ident"),
            Some(ident) => {
                let k = ident.to_string();
                match self.config.get(&k) {
                    None => return err(node, format!("unknown config key '{}'", k)),
                    Some(v) => v,
                }
            }
        })
    }
}
