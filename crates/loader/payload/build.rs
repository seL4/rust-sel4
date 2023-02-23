use std::env;
use std::fs;
use std::path::PathBuf;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use loader_payload_at_build_time as regions;

fn content_const_ident(i: usize) -> Ident {
    format_ident!("CONTENT_{}", i)
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let (payload, content_slices) = regions::get_split();

    let mut actual_content_toks = quote!();
    for (i, content) in content_slices.iter().enumerate() {
        let fname = format!("content.{}.bin", i);
        let out_path = PathBuf::from(&out_dir).join(&fname);
        fs::write(out_path, content).unwrap();
        let ident = content_const_ident(i);
        actual_content_toks.extend(quote! {
            const #ident: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), concat!("/", #fname)));
        });
    }

    let payload_info_toks = format!("{:?}", payload.info)
        .parse::<TokenStream>()
        .unwrap();

    let mut regions = vec![];
    for content in payload.data.iter() {
        let phys_addr_range_start = content.phys_addr_range.start;
        let phys_addr_range_end = content.phys_addr_range.end;
        let content_toks = match content.content {
            Some(i) => {
                let ident = content_const_ident(i);
                quote!(Some(#ident))
            }
            None => {
                quote!(None)
            }
        };
        regions.push(quote! {
            Region {
                phys_addr_range: #phys_addr_range_start..#phys_addr_range_end,
                content: #content_toks,
            }
        });
    }

    let toks = quote! {
        #actual_content_toks

        pub const PAYLOAD: Payload<&'static [Region<&'static [u8]>]> = Payload {
            info: #payload_info_toks,
            data: &[#(#regions,)*],
        };
    };
    let out_path = PathBuf::from(&out_dir).join("gen.rs");
    fs::write(out_path, format!("{}", toks)).unwrap();
}
