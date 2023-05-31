#![feature(int_roundings)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::fs;

use anyhow::Result;

use capdl_types::{FileContent, Footprint, Spec};

mod args;
mod render_elf;
mod reserialize_spec;

use args::Args;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectNamesLevel {
    All,
    JustTCBs,
    None,
}

fn main() -> Result<()> {
    let args = Args::parse()?;
    if args.verbose {
        eprintln!("{:#?}", args);
    }

    let initializer_elf = fs::read(&args.initializer_elf_path)?;
    let spec_json = fs::read(&args.spec_json_path)?;
    let fill_dir_path = &args.fill_dir_path;
    let out_file_path = &args.out_file_path;
    let object_names_level = &args.object_names_level;

    let input_spec: Spec<String, FileContent> = serde_json::from_slice(&spec_json).unwrap();

    let (final_spec, serialized_spec) =
        reserialize_spec::reserialize_spec(&input_spec, fill_dir_path, object_names_level);

    let footprint = final_spec.total_footprint();

    // TODO make configurable
    let heap_size = footprint * 2 + 16 * 4096;

    if args.verbose {
        eprintln!("footprint: {}", footprint);
        eprintln!("heap size: {}", heap_size / 4096);
    }

    let armed_initializer_elf = render_elf::render_elf(&initializer_elf, &serialized_spec, heap_size);

    fs::write(out_file_path, &armed_initializer_elf)?;
    Ok(())
}
