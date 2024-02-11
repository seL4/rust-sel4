use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use sel4_config_generic_types::{Configuration, Value};

pub fn generate_consts(config: &Configuration) -> TokenStream {
    let items = config.iter().map(|(k, v)| {
        let k = format_ident!("{}", k);
        let tv = match v {
            Value::Bool(v) => {
                quote! {
                    bool = #v
                }
            }
            Value::String(v) => {
                quote! {
                    &str = #v
                }
            }
        };
        quote! {
            pub const #k: #tv;
        }
    });

    quote! {
        #(#items)*
    }
}
