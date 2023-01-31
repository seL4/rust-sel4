#![feature(exit_status_error)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use which::which;

const CAPDL_EMBED_NAMES_ENV: &str = "CAPDL_EMBED_NAMES";

fn main() {
    let embed_names = env::var(CAPDL_EMBED_NAMES_ENV)
        .map(|x| x.parse::<i32>().unwrap())
        .unwrap_or(1)
        == 1;
    let deflate_fill = cfg!(feature = "deflate");
    let spec = capdl_embedded_spec_serialized::get();
    let (embedded_spec, aux_files) = capdl_embed_spec::embed(&spec, embed_names, deflate_fill);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let spec_out_path = out_dir.join("spec.rs");
    fs::write(&spec_out_path, format!("{}", embedded_spec)).unwrap();
    for (fname, content) in &aux_files {
        fs::write(out_dir.join(fname), content).unwrap();
    }

    println!("cargo:rerun-if-env-changed={}", CAPDL_EMBED_NAMES_ENV);

    // HACK
    // TODO not formatting entire file
    if let Some(rustfmt) = env::var("RUSTFMT")
        .map(PathBuf::from)
        .ok()
        .or_else(|| which("rustfmt").ok())
    {
        let output = Command::new(rustfmt).arg(&spec_out_path).output().unwrap();
        eprint!("{}", str::from_utf8(&output.stderr).unwrap());
        output.status.exit_ok().unwrap();
    }
}
