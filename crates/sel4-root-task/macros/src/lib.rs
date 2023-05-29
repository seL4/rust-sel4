use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn root_task(attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_ = quote!(sel4_root_task);
    let module_path = quote!(declare_root_task);
    let item = parse_macro_input!(item as syn::ItemFn);
    let ident = &item.sig.ident;
    let attr = TokenStream2::from(attr);
    let extra = if attr.is_empty() {
        quote!()
    } else {
        quote!(, #attr)
    };
    quote! {
        ::#crate_::#module_path!(main = #ident #extra);

        #item
    }
    .into()
}
