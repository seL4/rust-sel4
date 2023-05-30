use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;

/// Declares the initialization function, stack size, and, optionally, heap and heap size.
///
/// The syntax is:
///
/// ```rust
/// #[protection_domain($($key:ident = $value:expr),* $(,)?)]
/// fn init() -> impl Handler {
///    // ...
/// }
/// ```
///
/// Where the possible keys are:
///   - `stack_size`: Sets the stack size. Defaults to `0x4000`.
///   - `heap_size`: Declares a `#[global_allocator]` implemented using Dlmalloc and a
///     statically-allocated heap. Optional.
///
/// The function to which the attribute is applied will be used to initialize the protection domain.
/// It must satisfy `FnOnce() -> T where T: Handler`.
///
/// This macro is a thin wrapper around `sel4cp::declare_protection_domain`. The following are
/// equivalent:
///
/// ```rust
/// #[protection_domain(stack_size = 0x12000, heap_size = 0x34000)]
/// fn init() -> impl Handler {
///     // ...
/// }
/// ```
///
/// ```rust
/// declare_protection_domain! {
///     init = my_init,
///     stack_size = 0x12000,
///     heap_size = 0x34000,
/// }
///
/// fn init() -> impl Handler {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn protection_domain(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    let ident = &item.sig.ident;
    let attr = TokenStream2::from(attr);
    quote! {
        ::sel4cp::declare_protection_domain!(init = #ident, #attr);

        #item
    }
    .into()
}
