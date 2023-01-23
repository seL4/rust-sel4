#![feature(rustc_private)]

extern crate rustc_target;

use std::fs;
use std::path::Path;

use rustc_target::json::ToJson;
use rustc_target::spec::PanicStrategy;
use rustc_target::spec::Target;
use rustc_target::spec::TargetTriple;

// use std::collections::BTreeMap;

// use rustc_target::spec::Cc;
// use rustc_target::spec::Lld;

use clap::{App, Arg};

fn builtin(triple: &str) -> Target {
    Target::expect_builtin(&TargetTriple::from_triple(triple))
}

fn apply_common(target: &mut Target) {
    let options = &mut target.options;
    options.is_builtin = false;
    options.env = "sel4".into();
    options.exe_suffix = ".elf".into();
    options.has_thread_local = true;
    options.eh_frame_header = true;
    options.panic_strategy = PanicStrategy::Unwind;
}

fn aarch64_sel4() -> Target {
    let mut target = builtin("aarch64-unknown-none");
    apply_common(&mut target);
    target.llvm_target = "aarch64-none-elf".into();
    target
}

fn riscv64imac_sel4() -> Target {
    let mut target = builtin("riscv64imac-unknown-none-elf");
    apply_common(&mut target);
    target
}

fn x86_64_sel4() -> Target {
    let mut target = builtin("x86_64-unknown-none");
    apply_common(&mut target);
    let options = &mut target.options;
    options.position_independent_executables = false;
    options.static_position_independent_executables = false;
    // options.code_model = None; // TODO
    target
}

fn targets() -> Vec<(&'static str, Target)> {
    vec![
        ("aarch64-sel4", aarch64_sel4()),
        ("riscv64imac-sel4", riscv64imac_sel4()),
        ("x86_64-sel4", x86_64_sel4()),
    ]
}

fn write(target_dir: impl AsRef<Path>, target_name: &str, target: &Target) -> std::io::Result<()> {
    let path = target_dir.as_ref().join(format!("{}.json", target_name));
    let contents = format!("{:#}\n", target.to_json());
    fs::write(path, contents)
}

fn main() -> std::io::Result<()> {
    let matches = App::new("")
        .arg(Arg::from_usage("<target_dir>"))
        .get_matches();
    let target_dir = matches.value_of("target_dir").unwrap();
    for (target_name, target) in targets() {
        write(target_dir, target_name, &target)?;
    }
    Ok(())
}
