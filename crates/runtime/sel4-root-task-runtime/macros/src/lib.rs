extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_ = quote!(sel4_root_task_runtime);
    let module_path = quote!(declare_main);
    let item = parse_macro_input!(item as syn::ItemFn);
    let ident = &item.sig.ident;
    quote! {
        ::#crate_::#module_path!(#ident);

        #item
    }
    .into()
}
