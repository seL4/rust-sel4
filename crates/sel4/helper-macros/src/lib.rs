use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CapType)]
pub fn cap_type_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    cap_type_derive_impl(&ast)
}

fn cap_type_derive_impl(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name_str = name.to_string();
    quote! {
        impl CapType for #name {
            const NAME: &'static str = #name_str;
        }
    }
    .into()
}
