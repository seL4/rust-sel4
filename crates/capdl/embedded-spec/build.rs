#![feature(exit_status_error)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::str;

use capdl_embed_spec::IncludeObjectNamesConfig;
use sel4_rustfmt_helper::Rustfmt;

const CAPDL_EMBED_NAMES_ENV: &str = "CAPDL_EMBED_NAMES";

fn main() {
    let include_object_names = match env::var(CAPDL_EMBED_NAMES_ENV)
        .map(|x| x.parse::<i32>().unwrap())
        .unwrap_or(1)
    {
        0 => IncludeObjectNamesConfig::None,
        1 => IncludeObjectNamesConfig::JustTCBs,
        2 => IncludeObjectNamesConfig::All,
        n => panic!("unexpected value for {}: {}", CAPDL_EMBED_NAMES_ENV, n),
    };

    let deflate_fill = cfg!(feature = "deflate");

    let spec = capdl_embedded_spec_serialized::get();

    let (embedded_spec, aux_files) = capdl_embed_spec::Config {
        include_object_names,
        deflate_fill,
    }
    .embed(&spec);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let spec_out_path = out_dir.join("spec.rs");
    fs::write(&spec_out_path, format!("{}", embedded_spec)).unwrap();
    for (fname, content) in &aux_files {
        fs::write(out_dir.join(fname), content).unwrap();
    }

    println!("cargo:rerun-if-env-changed={}", CAPDL_EMBED_NAMES_ENV);

    Rustfmt::detect().format(&spec_out_path);
}
