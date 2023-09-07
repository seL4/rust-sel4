#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_target;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use rustc_target::json::ToJson;
use rustc_target::spec::{Cc, CodeModel, LinkerFlavor, Lld, PanicStrategy, Target, TargetTriple};

use clap::{Arg, ArgAction, Command};

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
    Riscv32,
    X86_64,
}

impl Config {
    fn target_spec(&self) -> Target {
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
                        // TODO
                        // Consider a configuration with --omagic/--nmagic/similar to further reduce
                        // wasted space in cases where segments are mapped without regards for
                        // permissions. --no-rosegment could be a good place to start.
                    ],
                )]);
                target
            }
            Arch::Riscv64 => builtin("riscv64imac-unknown-none-elf"),
            Arch::Riscv32 => builtin("riscv32imac-unknown-none-elf"),
            Arch::X86_64 => {
                let mut target = builtin("x86_64-unknown-none");
                let options = &mut target.options;
                options.position_independent_executables = false;
                options.static_position_independent_executables = false;
                options.code_model = Some(CodeModel::Small);
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
            Self::Riscv32 => "riscv32imac".to_owned(),
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
        vec![Self::AArch64, Self::Riscv64, Self::Riscv32, Self::X86_64]
    }
}

fn builtin(triple: &str) -> Target {
    Target::expect_builtin(&TargetTriple::from_triple(triple))
}

fn all_target_specs() -> BTreeMap<String, Target> {
    Config::all()
        .into_iter()
        .map(|config| (config.name(), config.target_spec()))
        .collect::<BTreeMap<_, _>>()
}

fn do_list() {
    for target_name in all_target_specs().keys() {
        println!("{}", target_name)
    }
}

fn do_write(
    target_dir: impl AsRef<Path>,
    optional_targets: Option<Vec<String>>,
) -> std::io::Result<()> {
    let all_targets = all_target_specs();
    let these_targets =
        optional_targets.unwrap_or_else(|| all_target_specs().keys().cloned().collect::<Vec<_>>());
    for target_name in &these_targets {
        let target_spec = all_targets.get(target_name).unwrap();
        write_one(&target_dir, target_name, target_spec)?;
    }
    Ok(())
}

fn write_one(
    target_dir: impl AsRef<Path>,
    target_name: &str,
    target_spec: &Target,
) -> std::io::Result<()> {
    let path = target_dir.as_ref().join(format!("{}.json", target_name));
    let contents = format!("{:#}\n", target_spec.to_json());
    fs::write(path, contents)
}

fn main() -> std::io::Result<()> {
    let matches = Command::new("")
        .subcommand_required(true)
        .subcommand(Command::new("list"))
        .subcommand(
            Command::new("write")
                .arg(
                    Arg::new("target_dir")
                        .long("target-dir")
                        .short('d')
                        .value_name("TARGET_DIR")
                        .required(true),
                )
                .arg(
                    Arg::new("targets")
                        .long("target")
                        .short('t')
                        .value_name("TARGET")
                        .action(ArgAction::Append),
                )
                .arg(Arg::new("all").long("all").action(ArgAction::SetTrue)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("list", _)) => {
            do_list();
        }
        Some(("write", sub_matches)) => {
            let target_dir = sub_matches.value_of("target_dir").unwrap();
            let targets = sub_matches
                .get_many::<String>("targets")
                .map(|many| many.cloned().collect::<Vec<_>>())
                .unwrap_or_else(Vec::new);
            let all = *sub_matches.get_one::<bool>("all").unwrap();
            let optional_targets = if all { None } else { Some(targets) };
            do_write(target_dir, optional_targets)?;
        }
        _ => {
            unreachable!()
        }
    }

    Ok(())
}
