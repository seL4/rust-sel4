#![feature(int_roundings)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::fs;

use anyhow::Result;

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

    let loader_elf = fs::read(&args.loader_elf_path)?;
    let spec_json = fs::read(&args.spec_json_path)?;
    let fill_dir_path = &args.fill_dir_path;
    let out_file_path = &args.out_file_path;
    let object_names_level = &args.object_names_level;

    let serialized_spec =
        reserialize_spec::reserialize_spec(&spec_json, fill_dir_path, object_names_level);

    let armed_loader_elf = render_elf::render_elf(&loader_elf, &serialized_spec, 128 * 4096);

    fs::write(out_file_path, &armed_loader_elf)?;
    Ok(())
}
