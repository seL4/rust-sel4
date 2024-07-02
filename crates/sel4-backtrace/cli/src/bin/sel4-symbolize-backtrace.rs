//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;

use clap::{arg, Command};

use sel4_backtrace_addr2line_context_helper::new_context;
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
    let elf_file_contents = fs::read(elf_file_path).unwrap();
    let obj = object::File::parse(&*elf_file_contents).unwrap();
    let ctx = new_context(&obj).unwrap();
    println!("backtrace: {}", elf_file_path);
    let mut s = String::new();
    bt.symbolize(&ctx, &mut s).unwrap();
    print!("{}", s);
}
