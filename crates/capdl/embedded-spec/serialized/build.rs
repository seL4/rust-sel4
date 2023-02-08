#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::collections::BTreeSet;
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

use quote::quote;

use capdl_types::*;

const CAPDL_SPEC_FILE_ENV: &str = "CAPDL_SPEC_FILE";
const CAPDL_FILL_DIR_ENV: &str = "CAPDL_FILL_DIR";

// NOTE
// we don't use fs::copy because sources may have incompatible permissions

fn main() {
    let spec_file_path = env::var(CAPDL_SPEC_FILE_ENV).unwrap();
    let fill_dir_path = env::var(CAPDL_FILL_DIR_ENV).unwrap();

    println!("cargo:rerun-if-env-changed={}", CAPDL_SPEC_FILE_ENV);
    println!("cargo:rerun-if-env-changed={}", CAPDL_FILL_DIR_ENV);
    println!("cargo:rerun-if-changed={}", spec_file_path);

    // TODO is this excessively coarse?
    println!("cargo:rerun-if-changed={}", fill_dir_path);

    let out_dir = env::var("OUT_DIR").unwrap();

    io::copy(
        &mut File::open(&spec_file_path).unwrap(),
        &mut File::create(PathBuf::from(&out_dir).join("spec.json")).unwrap(),
    )
    .unwrap();

    let spec: Spec<String, FileContent> =
        serde_json::from_reader(File::open(spec_file_path).unwrap()).unwrap();

    let mut files = BTreeSet::new();
    spec.traverse_fill(|fill| {
        if !files.contains(&fill.file) {
            files.insert(fill.file.to_owned());
        }
        Ok::<(), !>(())
    })
    .into_ok();

    let toks = files.iter().map(|file| {
        let encoded = format!("file.{}.bin", hex::encode(file));
        io::copy(
            &mut File::open(PathBuf::from(&fill_dir_path).join(file)).unwrap(),
            &mut File::create(PathBuf::from(&out_dir).join(&encoded)).unwrap(),
        )
        .unwrap();
        quote! {
            (#file, include_bytes!(concat!(env!("OUT_DIR"), concat!("/", #encoded))))
        }
    });
    let toks = quote! {
        &[#(#toks,)*]
    };

    fs::write(
        PathBuf::from(&out_dir).join("files.rs"),
        format!("{}", toks),
    )
    .unwrap()
}
