#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_target;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use rustc_target::json::ToJson;
use rustc_target::spec::Cc;
use rustc_target::spec::LinkerFlavor;
use rustc_target::spec::Lld;
use rustc_target::spec::PanicStrategy;
use rustc_target::spec::Target;
use rustc_target::spec::TargetTriple;

use clap::{App, Arg};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Config {
    arch: Arch,
    cp: bool,
    minimal: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Arch {
    AArch64,
    Riscv64,
    X86_64,
}

impl Config {
    fn target(&self) -> Target {
        let mut target = match &self.arch {
            Arch::AArch64 => {
                let mut target = builtin("aarch64-unknown-none");
                target.llvm_target = "aarch64-none-elf".into();
                let options = &mut target.options;
                options.pre_link_args = BTreeMap::from_iter([(
                    LinkerFlavor::Gnu(Cc::No, Lld::Yes),
                    vec![
                        // Determines p_align. Default is 64K, which results in wasted space when
                        // the ELF file is loaded as a single contiguous region as it is in the case
                        // of a root task.
                        "-z".into(),
                        "max-page-size=4096".into(),
                    ],
                )]);
                target
            }
            Arch::Riscv64 => builtin("riscv64imac-unknown-none-elf"),
            Arch::X86_64 => {
                let mut target = builtin("x86_64-unknown-none");
                let options = &mut target.options;
                options.position_independent_executables = false;
                options.static_position_independent_executables = false;
                // options.code_model = None; // TODO
                target
            }
        };

        {
            let options = &mut target.options;
            options.is_builtin = false;
            options.env = "sel4".into();
            options.exe_suffix = ".elf".into();
            options.eh_frame_header = !self.minimal;
        }

        if self.cp {
            let options = &mut target.options;
            options.link_script =
                Some("__sel4_ipc_buffer_obj = (_end + 4096 - 1) & ~(4096 - 1);".into());
        }

        if !self.minimal {
            let options = &mut target.options;
            options.has_thread_local = true;
            options.panic_strategy = PanicStrategy::Unwind;
        }

        target
    }

    fn filter(&self) -> bool {
        !self.cp || self.arch.cp_support()
    }

    fn name(&self) -> String {
        let mut name = self.arch.name();
        name.push_str("-sel4");
        if self.cp {
            name.push_str("cp");
        }
        if self.minimal {
            name.push_str("-minimal");
        }
        name
    }

    fn all() -> Vec<Self> {
        let mut all = vec![];
        let all_bools = &[true, false];
        for arch in Arch::all() {
            for cp in all_bools.iter().copied() {
                for minimal in all_bools.iter().copied() {
                    let config = Self { arch, cp, minimal };
                    if config.filter() {
                        all.push(config);
                    }
                }
            }
        }
        all
    }
}

impl Arch {
    fn name(&self) -> String {
        match self {
            Self::AArch64 => "aarch64".to_owned(),
            Self::Riscv64 => "riscv64imac".to_owned(),
            Self::X86_64 => "x86_64".to_owned(),
        }
    }

    fn cp_support(&self) -> bool {
        match self {
            Self::AArch64 => true,
            _ => false,
        }
    }

    fn all() -> Vec<Self> {
        vec![Self::AArch64, Self::Riscv64, Self::X86_64]
    }
}

fn builtin(triple: &str) -> Target {
    Target::expect_builtin(&TargetTriple::from_triple(triple))
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
    for config in Config::all().iter() {
        write(target_dir, &config.name(), &config.target())?;
    }
    Ok(())
}
