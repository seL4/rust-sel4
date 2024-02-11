use std::env;
use std::fs;
use std::path::PathBuf;

use quote::{format_ident, quote};

use sel4_config_data::get_kernel_config;
use sel4_config_generic_types::Value;
use sel4_rustfmt_helper::Rustfmt;

fn main() {
    let config = get_kernel_config();

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

    let toks = quote! {
        #(#items)*
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("gen.rs");
    fs::write(&out_path, format!("{}", toks)).unwrap();
    Rustfmt::detect().format(&out_path);
}
