use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, spanned::Spanned, Token};

use sel4_config_generic_types::Configuration;

use crate::{parse_or_return, Evaluator};

pub fn cfg_if_impl(config: &Configuration, toks: TokenStream) -> TokenStream {
    let evaluator = Evaluator::new(config);
    let input = parse_or_return!(toks as CfgIfInput);
    for branch_with_condition in input.branches_with_conditions.iter() {
        match evaluator.eval_nested_meta(&branch_with_condition.condition) {
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

struct CfgIfInput {
    branches_with_conditions: Vec<BranchWithCondition>,
    trailing_branch_without_condition: Option<TokenStream>,
}

impl syn::parse::Parse for CfgIfInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut branches_with_conditions = vec![input.parse()?];
        while input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            if input.peek(Token![if]) {
                branches_with_conditions.push(input.parse()?);
            } else {
                break;
            }
        }
        let trailing_branch_without_condition = if input.is_empty() {
            None
        } else {
            Some(input.call(parse_branch)?)
        };
        Ok(Self {
            branches_with_conditions,
            trailing_branch_without_condition,
        })
    }
}

struct BranchWithCondition {
    condition: syn::NestedMeta,
    branch: TokenStream,
}

impl syn::parse::Parse for BranchWithCondition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let condition = input.call(parse_condition)?;
        let branch = input.call(parse_branch)?;
        Ok(Self { condition, branch })
    }
}

type Condition = syn::NestedMeta;

const CFG: &str = "cfg";

fn parse_condition(input: syn::parse::ParseStream) -> syn::Result<Condition> {
    input.parse::<Token![if]>()?;
    let attrs = syn::Attribute::parse_outer(input)?;
    let attr = match attrs.len() {
        0 => return Err(syn::Error::new(input.span(), "expected attribute")),
        1 => &attrs[0],
        _ => {
            return Err(syn::Error::new(
                attrs[1].span(),
                "expected just one attribute",
            ))
        }
    };
    if !attr.path.is_ident(&format_ident!("{}", CFG)) {
        return Err(syn::Error::new(attr.span(), format!("expected '{}'", CFG)));
    }
    attr.parse_args()
}

type Branch = TokenStream;

fn parse_branch(input: syn::parse::ParseStream) -> syn::Result<Branch> {
    let branch;
    syn::braced!(branch in input);
    branch.parse()
}
