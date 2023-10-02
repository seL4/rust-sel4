use std::fs;
use std::path::Path;

use clap::{Arg, ArgAction, Command};
use glob::glob;

fn main() {
    let matches = Command::new("")
        .arg(
            Arg::new("dir")
                .short('d')
                .required(true)
                .action(ArgAction::Append),
        )
        .get_matches();
    let dirs = matches.get_many::<String>("dir").unwrap();
    for d in dirs {
        for f in glob(&format!("{}/**/*.pbf", d))
            .unwrap()
            .map(Result::unwrap)
        {
            test_on_path(f);
        }
    }
}

fn test_on_path(f: impl AsRef<Path>) {
    println!("parsing '{}'", f.as_ref().display());
    let text = fs::read_to_string(f).unwrap();
    let file = sel4_bitfield_parser::parse(&text);
    println!("{:#?}", file);
}
