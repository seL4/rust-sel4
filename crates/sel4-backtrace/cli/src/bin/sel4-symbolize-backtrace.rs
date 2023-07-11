use std::fs::File;

use addr2line::Context;
use clap::{App, Arg};
use memmap::Mmap;

use sel4_backtrace_types::Backtrace;

fn main() {
    let matches = App::new("")
        .arg(Arg::from_usage("-f --file=[ELF]"))
        .arg(Arg::from_usage("<raw_backtrace>"))
        .get_matches();
    let bt_hex = matches.value_of("raw_backtrace").unwrap();
    let bt = Backtrace::<Option<String>>::recv(&hex::decode(bt_hex).unwrap()).unwrap();
    let elf_file_path = matches
        .value_of("file")
        .or(bt.preamble.image.as_ref().map(String::as_str))
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
