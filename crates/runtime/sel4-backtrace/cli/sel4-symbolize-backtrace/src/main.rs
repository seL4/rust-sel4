use std::fs::File;

use addr2line::Context;
use clap::{App, Arg};
use fallible_iterator::FallibleIterator;
use gimli::read::Reader;
use memmap::Mmap;

use sel4_backtrace_types::Backtrace;

// TODO handle inlining better (see TODOs scattered throughout this file)

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
    show_backtrace(ctx, &bt, elf_file_path).unwrap();
}

fn show_backtrace<R: Reader>(
    ctx: Context<R>,
    bt: &Backtrace<Option<String>>,
    elf_file_path: &str,
) -> Result<(), gimli::Error> {
    println!("backtrace: {}", elf_file_path);
    if let Some(ref err) = bt.postamble.error {
        println!("    error: {:?}", err);
    }
    for (i, entry) in bt.entries.iter().enumerate() {
        let mut first = true;
        let frame = &entry.stack_frame;
        // TODO
        // let mut seen = false;
        // let initial_location = ctx.find_location(frame.ip as u64)?;
        ctx.find_frames(frame.ip as u64)?.for_each(|inner_frame| {
            if first {
                print!(" {:4}:  {:#18x} - ", i, frame.ip);
            } else {
                print!(" {:4}   {:18  }   ", "", "");
            }
            // TODO
            // if inner_frame.location == frame {
            //     seen = true;
            // }
            match inner_frame.function {
                Some(f) => {
                    // TODO
                    // let raw_name = f.raw_name()?;
                    // let demangled = demangle(&raw_name);
                    let demangled = f.demangle()?;
                    print!("{}", demangled)
                }
                None => print!("<unknown>"),
            }
            print!("\n");
            // TODO
            // if let Some(loc) = inner_frame.location {
            //     println!("      {:18}       at {}", "", fmt_location(loc));
            // }
            first = false;
            Ok(())
        })?;
        // TODO this isn't accurate
        if let Some(loc) = ctx.find_location(frame.ip as u64)? {
            println!("      {:18}       at {}", "", fmt_location(loc));
        }
        // TODO
        // if !seen {
        //     print!("      ");
        //     print!("warning: initial location missing: {}", initial_location);
        //     print!("\n");
        // }
    }
    Ok(())
}

fn fmt_location(loc: addr2line::Location) -> String {
    format!(
        "{} {},{}",
        loc.file.unwrap_or("<unknown>"),
        loc.line
            .map(|x| x.to_string())
            .unwrap_or(String::from("<unknown>")),
        loc.column
            .map(|x| x.to_string())
            .unwrap_or(String::from("<unknown>")),
    )
}
