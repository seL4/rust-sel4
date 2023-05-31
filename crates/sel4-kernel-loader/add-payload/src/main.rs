#![feature(int_roundings)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::fs;

use anyhow::Result;

mod args;
mod render_elf;
mod serialize_payload;

use args::Args;

fn main() -> Result<()> {
    let args = Args::parse()?;
    if args.verbose {
        eprintln!("{:#?}", args);
    }

    let loader_bytes = fs::read(&args.loader_path)?;
    let out_file_path = &args.out_file_path;

    let serialized_payload = serialize_payload::serialize_payload(
        &args.kernel_path,
        &args.app_path,
        &args.dtb_path,
        &args.platform_info_path,
    );

    if args.verbose {}

    let loader_with_payload_bytes = render_elf::render_elf(&loader_bytes, &serialized_payload);

    fs::write(out_file_path, &loader_with_payload_bytes)?;
    Ok(())
}
