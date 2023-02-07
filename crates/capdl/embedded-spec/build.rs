use std::env;
use std::fs;
use std::path::PathBuf;
use std::str;

use capdl_embed_spec::ObjectNamesLevel;
use sel4_rustfmt_helper::Rustfmt;

const CAPDL_OBJECT_NAMES_LEVEL_ENV: &str = "CAPDL_OBJECT_NAMES_LEVEL";

fn main() {
    let object_names_level = env::var(CAPDL_OBJECT_NAMES_LEVEL_ENV)
        .map(|val| match val.parse::<usize>().unwrap() {
            0 => ObjectNamesLevel::None,
            1 => ObjectNamesLevel::JustTCBs,
            2 => ObjectNamesLevel::All,
            n => panic!(
                "unexpected value for {}: {}",
                CAPDL_OBJECT_NAMES_LEVEL_ENV, n
            ),
        })
        .unwrap_or(ObjectNamesLevel::JustTCBs);

    let deflate_fill = cfg!(feature = "deflate");

    let spec = capdl_embedded_spec_serialized::get();

    let (embedded_spec, aux_files) = capdl_embed_spec::Config {
        object_names_level,
        deflate_fill,
    }
    .embed(&spec);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let spec_out_path = out_dir.join("spec.rs");
    fs::write(&spec_out_path, format!("{}", embedded_spec)).unwrap();
    for (fname, content) in &aux_files {
        fs::write(out_dir.join(fname), content).unwrap();
    }

    println!(
        "cargo:rerun-if-env-changed={}",
        CAPDL_OBJECT_NAMES_LEVEL_ENV
    );

    Rustfmt::detect().format(&spec_out_path);
}
