use std::env;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn env_literal(var: TokenStream) -> TokenStream {
    let var = syn::parse::<syn::LitStr>(var).unwrap().value();
    match env::var(var).ok() {
        Some(val) => {
            let val = syn::parse_str::<syn::Lit>(&val).unwrap();
            quote!(Some(#val))
        }
        None => {
            quote!(None)
        }
    }
    .into()
}
