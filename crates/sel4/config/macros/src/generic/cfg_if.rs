//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Token,
    parse::{ParseStream, Parser},
    spanned::Spanned,
};

use super::{Condition, MacroImpls};

impl MacroImpls<'_> {
    pub fn cfg_if_impl(&self, toks: TokenStream) -> TokenStream {
        let parser = move |parse_stream: ParseStream| {
            parse_cfg_if_input(self.synthetic_attr(), parse_stream)
        };
        let input = match parser.parse2(toks) {
            Ok(parsed) => parsed,
            Err(err) => {
                return err.to_compile_error();
            }
        };
        for branch_with_condition in input.branches_with_conditions.iter() {
            match self.eval(&branch_with_condition.condition) {
                Ok(pass) => {
                    if pass {
                        return branch_with_condition.branch.clone();
                    }
                }
                Err(err) => {
                    return err.render();
                }
            }
        }
        match &input.trailing_branch_without_condition {
            Some(branch) => branch.clone(),
            None => quote!(),
        }
    }
}

struct CfgIfInput {
    branches_with_conditions: Vec<BranchWithCondition>,
    trailing_branch_without_condition: Option<TokenStream>,
}

fn parse_cfg_if_input(
    synthetic_attr: &str,
    input: syn::parse::ParseStream,
) -> syn::Result<CfgIfInput> {
    let mut branches_with_conditions = vec![parse_branch_with_condition(synthetic_attr, input)?];
    while input.peek(Token![else]) {
        input.parse::<Token![else]>()?;
        if input.peek(Token![if]) {
            branches_with_conditions.push(parse_branch_with_condition(synthetic_attr, input)?);
        } else {
            break;
        }
    }
    let trailing_branch_without_condition = if input.is_empty() {
        None
    } else {
        Some(input.call(parse_branch)?)
    };
    Ok(CfgIfInput {
        branches_with_conditions,
        trailing_branch_without_condition,
    })
}

struct BranchWithCondition {
    condition: Condition,
    branch: TokenStream,
}

fn parse_branch_with_condition(
    synthetic_attr: &str,
    input: syn::parse::ParseStream,
) -> syn::Result<BranchWithCondition> {
    let condition = parse_condition(synthetic_attr, input)?;
    let branch = parse_branch(input)?;
    Ok(BranchWithCondition { condition, branch })
}

fn parse_condition(synthetic_attr: &str, input: syn::parse::ParseStream) -> syn::Result<Condition> {
    input.parse::<Token![if]>()?;
    let attrs = syn::Attribute::parse_outer(input)?;
    let attr = match attrs.len() {
        0 => return Err(input.error("expected attribute")),
        1 => &attrs[0],
        _ => {
            return Err(syn::Error::new(
                attrs[1].span(),
                "expected just one attribute",
            ));
        }
    };
    if !attr.path().is_ident(synthetic_attr) {
        return Err(syn::Error::new(
            attr.span(),
            format!("expected '{synthetic_attr}'"),
        ));
    }
    attr.parse_args()
}

type Branch = TokenStream;

fn parse_branch(input: syn::parse::ParseStream) -> syn::Result<Branch> {
    let branch;
    syn::braced!(branch in input);
    branch.parse()
}
