//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

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
    context: Context,
    minimal: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Arch {
    AArch64,
    Armv7a,
    RiscV64(RiscVArch),
    RiscV32(RiscVArch),
    X86_64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
enum RiscVArch {
    IMAC,
    IMAFC,
    GC,
}

impl RiscVArch {
    fn arch_suffix_for_target_name(&self) -> String {
        match self {
            Self::IMAFC => "imafc".to_owned(),
            Self::IMAC => "imac".to_owned(),
            Self::GC => "gc".to_owned(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Context {
    RootTask,
    Microkit { resettable: bool },
}

impl Config {
    fn target_spec(&self) -> Target {
        let mut target = match &self.arch {
            Arch::AArch64 => {
                let mut target = builtin("aarch64-unknown-none");
                // target.llvm_target = "aarch64-none-elf".into(); // TODO why or why not?
                let options = &mut target.options;
                let linker_flavor = LinkerFlavor::Gnu(Cc::No, Lld::Yes);
                assert_eq!(options.linker_flavor, linker_flavor);
                options.pre_link_args = BTreeMap::from_iter([(
                    linker_flavor,
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
            Arch::Armv7a => builtin("armv7a-none-eabi"),
            Arch::RiscV64(riscv_arch) => builtin(&format!(
                "riscv64{}-unknown-none-elf",
                riscv_arch.arch_suffix_for_target_name()
            )),
            Arch::RiscV32(riscv_arch) => builtin(&format!(
                "riscv32{}-unknown-none-elf",
                riscv_arch.arch_suffix_for_target_name()
            )),
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

        if let Context::Microkit { resettable } = &self.context {
            let mut linker_script = String::new();
            if *resettable {
                linker_script.push_str(include_str!("microkit-resettable.lds"));
            }
            linker_script.push_str("__sel4_ipc_buffer_obj = (__ehdr_start & ~(4096 - 1)) - 4096;");
            let options = &mut target.options;
            options.link_script = Some(linker_script.into());
        }

        if !self.minimal {
            let options = &mut target.options;
            options.has_thread_local = true;
            if self.arch.unwinding_support() {
                options.panic_strategy = PanicStrategy::Unwind;
            }
        }

        target
    }

    fn filter(&self) -> bool {
        !self.context.is_microkit() || self.arch.microkit_support()
    }

    fn name(&self) -> String {
        let mut name = self.arch.name();
        name.push_str("-sel4");
        if let Context::Microkit { resettable } = &self.context {
            name.push_str("-microkit");
            if *resettable {
                name.push_str("-resettable");
            }
        }
        if self.minimal {
            name.push_str("-minimal");
        }
        name
    }

    fn all() -> Vec<Self> {
        let mut all = vec![];
        for arch in Arch::all() {
            for context in Context::all() {
                for minimal in [true, false] {
                    let config = Self {
                        arch,
                        context,
                        minimal,
                    };
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
            Self::Armv7a => "armv7a".to_owned(),
            Self::RiscV64(riscv_arch) => {
                format!("riscv64{}", riscv_arch.arch_suffix_for_target_name())
            }
            Self::RiscV32(riscv_arch) => {
                format!("riscv32{}", riscv_arch.arch_suffix_for_target_name())
            }
            Self::X86_64 => "x86_64".to_owned(),
        }
    }

    fn microkit_support(&self) -> bool {
        matches!(self, Self::AArch64 | Self::RiscV64(_))
    }

    fn unwinding_support(&self) -> bool {
        // Due to lack of support (so far) for aarch32 in the 'unwinding' crate
        !matches!(self, Self::Armv7a)
    }

    fn all() -> Vec<Self> {
        vec![
            Self::AArch64,
            Self::Armv7a,
            Self::RiscV64(RiscVArch::IMAC),
            Self::RiscV64(RiscVArch::GC),
            Self::RiscV32(RiscVArch::IMAC),
            Self::RiscV32(RiscVArch::IMAFC),
            Self::X86_64,
        ]
    }
}

impl Context {
    fn is_microkit(&self) -> bool {
        matches!(self, Self::Microkit { .. })
    }

    fn all() -> Vec<Self> {
        let mut v = vec![];
        v.push(Self::RootTask);
        for resettable in [true, false] {
            v.push(Self::Microkit { resettable });
        }
        v
    }
}

fn builtin(triple: &str) -> Target {
    #[cfg_attr(not(target_spec_has_metadata), allow(unused_mut))]
    let mut target = Target::expect_builtin(&TargetTriple::from_triple(triple));
    #[cfg(target_spec_has_metadata)]
    {
        target.metadata = Default::default();
    }
    target
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
            let target_dir = sub_matches.get_one::<String>("target_dir").unwrap();
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
