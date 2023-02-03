#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::fs;

use anyhow::Result;

mod args;
mod render_elf;
mod reserialize_spec;

use args::Args;

fn main() -> Result<()> {
    let args = Args::parse()?;
    if args.verbose {
        eprintln!("{:#?}", args);
    }

    let loader_elf = fs::read(&args.loader_elf_path)?;
    let spec_json = fs::read(&args.spec_json_path)?;
    let fill_dir_path = &args.fill_dir_path;
    let out_file_path = &args.out_file_path;

    let serialized_spec = reserialize_spec::reserialize_spec(&spec_json, fill_dir_path);
    let armed_loader_elf = render_elf::render_elf(
        &loader_elf,
        &serialized_spec,
        "capdl_spec_start",
        "capdl_spec_size",
    );

    fs::write(out_file_path, &armed_loader_elf)?;
    Ok(())
}
