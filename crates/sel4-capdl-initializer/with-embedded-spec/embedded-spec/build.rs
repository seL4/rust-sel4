use std::env;
use std::fs;
use std::path::PathBuf;

use sel4_capdl_initializer_with_embedded_spec_build_env::get_embedding;
use sel4_rustfmt_helper::Rustfmt;

fn main() {
    // TODO
    // let deflate_fill = cfg!(feature = "deflate");

    let (embedding, footprint) = get_embedding();

    footprint.tell_cargo();

    let (embedded_spec, aux_files) = embedding.embed();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    {
        let spec_out_path = out_dir.join("spec.rs");
        fs::write(&spec_out_path, format!("{}", embedded_spec)).unwrap();
        Rustfmt::detect().format(&spec_out_path);
    }

    for (fname, content) in &aux_files {
        fs::write(out_dir.join(fname), content).unwrap();
    }
}
