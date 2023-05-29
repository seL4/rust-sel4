use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;

/// Declares the initialization function, stack size, and, optionally, heap and heap size.
///
/// This macro is a thin wrapper around `sel4cp::declare_protection_domain`. The following are equivalent:
///
/// ```rust
/// #[sel4cp::protection_domain(stack_size = 0x12000, heap_size = 0x34000)]
/// fn my_init() -> MyHandler {
///     // ...
/// }
/// ```
///
/// ```rust
/// sel4cp::declare_protection_domain! {
///     init = my_init,
///     stack_size = 0x12000,
///     heap_size = 0x34000,
/// }
///
/// fn my_init() -> MyHandler {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn protection_domain(attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_ = quote!(sel4cp);
    let module_path = quote!(declare_protection_domain);
    let item = parse_macro_input!(item as syn::ItemFn);
    let ident = &item.sig.ident;
    let attr = TokenStream2::from(attr);
    let extra = if attr.is_empty() {
        quote!()
    } else {
        quote!(, #attr)
    };
    quote! {
        ::#crate_::#module_path!(init = #ident #extra);

        #item
    }
    .into()
}
