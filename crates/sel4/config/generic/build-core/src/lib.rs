use std::error::Error;
use std::fs;
use std::path::Path;

use proc_macro2::TokenStream;
use quote::quote;

pub use sel4_config_generic_types::{Configuration, Value};

pub trait ConfigurationExt: Sized {
    fn read_json(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>>;

    fn generate_data_fragment(
        &self,
        fn_name_ident: TokenStream,
        helpers_module_path: TokenStream,
    ) -> TokenStream;
}

impl ConfigurationExt for Configuration {
    fn read_json(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let file = fs::File::open(path)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }

    fn generate_data_fragment(
        &self,
        fn_name_ident: TokenStream,
        helpers_module_path: TokenStream,
    ) -> TokenStream {
        let value_path = quote! {
            #helpers_module_path::Value
        };
        let to_string_path = quote! {
            #helpers_module_path::ToString::to_string
        };
        let insertions = self.iter().map(|(k, v)| {
            let v = match v {
                Value::Bool(v) => {
                    quote! {
                        #value_path::Bool(#v)
                    }
                }
                Value::String(v) => {
                    quote! {
                        #value_path::String(#to_string_path(#v))
                    }
                }
            };
            quote! {
                config.insert(
                    #to_string_path(#k),
                    #v,
                );
            }
        });
        quote! {
            #[allow(unused_mut)]
            fn #fn_name_ident() -> #helpers_module_path::Configuration {
                let mut config = #helpers_module_path::Configuration::empty();
                #(#insertions)*
                config
            }
        }
    }
}
