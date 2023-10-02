use std::fs::File;

use addr2line::Context;
use clap::{arg, Command};
use memmap::Mmap;

use sel4_backtrace_types::Backtrace;

fn main() {
    let matches = Command::new("")
        .arg(arg!(-f --file <ELF>))
        .arg(arg!(<raw_backtrace>))
        .get_matches();
    let bt_hex = matches.get_one::<String>("raw_backtrace").unwrap();
    let bt = Backtrace::<Option<String>>::recv(&hex::decode(bt_hex).unwrap()).unwrap();
    let elf_file_path = matches
        .get_one("file")
        .or(bt.preamble.image.as_ref())
        .expect("ELF file neither embedded nor provided");
    let elf_file = File::open(elf_file_path).unwrap();
    let map = unsafe { Mmap::map(&elf_file).unwrap() };
    let elf_obj = &object::File::parse(&*map).unwrap();
    let ctx = Context::new(elf_obj).unwrap();
    println!("backtrace: {}", elf_file_path);
    let mut s = String::new();
    bt.symbolize(&ctx, &mut s).unwrap();
    print!("{}", s);
}
