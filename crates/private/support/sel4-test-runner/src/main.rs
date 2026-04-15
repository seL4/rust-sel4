//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, iter};

use anyhow::{Error, ensure};
use clap::Parser;
use object::{Architecture, File, Object, ObjectSection as _, ObjectSymbol};
use tempfile::TempDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    exe: PathBuf,
    #[arg(long)]
    target_dir: PathBuf,
    #[arg(long)]
    object_sizes: PathBuf,
    #[arg(long)]
    kernel: Option<PathBuf>,
    #[arg(long)]
    microkit_sdk: Option<PathBuf>,
    #[arg(long)]
    microkit_board: Option<String>,
    #[arg(long)]
    microkit_config: Option<String>,
    #[arg(long, short = 'i')]
    interactive: bool,
    #[arg(long, short = 'n')]
    no_run: bool,
    #[arg(long)]
    simulate_script: PathBuf,
    #[arg(long, short = 't')]
    timeout: Option<u32>,
    #[arg(last = true)]
    simulate_args: Option<String>,
}

// TODO allow specifying timeout by embedding in file
const DEFAULT_TIMEOUT: u32 = 5;

#[derive(Debug)]
enum SeL4TestKind {
    RootTask,
    Microkit,
    CapDL,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let parent = cli.target_dir.join("runner");
    fs::create_dir_all(&parent)?;
    let mut d = TempDir::with_prefix_in("run-", parent)?;
    d.disable_cleanup(true);

    eprintln!("tmp:");
    eprintln!("{}", d.path().display());

    let exe = d.path().join(cli.exe.file_name().unwrap());
    fs::copy(&cli.exe, &exe)?;

    let data = fs::read(&exe)?;
    let file = object::File::parse(&*data)?;

    Runner {
        cli: &cli,
        d: d.path(),
        exe: &exe,
        file: &file,
    }
    .run()
}

struct Runner<'a> {
    cli: &'a Cli,
    d: &'a Path,
    exe: &'a Path,
    file: &'a File<'a>,
}

impl<'a> Runner<'a> {
    fn run(&self) -> anyhow::Result<()> {
        match self.get_sel4_test_kind() {
            None => self.run_not_sel4(),
            Some(kind) => {
                self.mk_resettable()?;
                let image = match kind {
                    SeL4TestKind::RootTask => self.mk_root_task_image(self.exe)?,
                    SeL4TestKind::Microkit => self.mk_microkit_image()?,
                    SeL4TestKind::CapDL => self.mk_capdl_image()?,
                };
                self.create_debugging_links()?;
                if self.cli.no_run {
                    println!("{}", self.d.display());
                } else if self.cli.interactive {
                    ensure!(
                        Command::new(&self.cli.simulate_script)
                            .arg(image)
                            .args(self.cli.simulate_args.iter())
                            .status()?
                            .success()
                    );
                } else {
                    let mut cmd = Command::new("timeout");
                    cmd.arg("-f");
                    cmd.arg(format!("{}s", self.cli.timeout.unwrap_or(DEFAULT_TIMEOUT)));
                    cmd.arg(&self.cli.simulate_script);
                    cmd.arg(image);
                    cmd.args(self.cli.simulate_args.iter());
                    sel4_test_sentinels_wrapper::default_sentinels()
                        .wrap(cmd)?
                        .success_ok()?;
                    println!();
                }
                Ok(())
            }
        }
    }

    fn create_debugging_links(&self) -> anyhow::Result<()> {
        let debug_bin = if let Some(kernel) = &self.cli.kernel {
            kernel.join("bin")
        } else if let Some(sdk) = &self.cli.microkit_sdk {
            sdk.join("board")
                .join(self.cli.microkit_board.as_ref().unwrap())
                .join(self.cli.microkit_config.as_ref().unwrap())
                .join("elf")
        } else {
            panic!()
        };
        unix::fs::symlink(debug_bin, self.d.join("debug-bin"))?;
        unix::fs::symlink(&self.cli.simulate_script, self.d.join("simulate"))?;
        Ok(())
    }

    fn get_sel4_test_kind(&self) -> Option<SeL4TestKind> {
        if self.check_sel4_test_kind("sel4_test_kind_root_task") {
            Some(SeL4TestKind::RootTask)
        } else if self.check_sel4_test_kind("sel4_test_kind_microkit") {
            Some(SeL4TestKind::Microkit)
        } else if self.check_sel4_test_kind("sel4_test_kind_capdl") {
            Some(SeL4TestKind::CapDL)
        } else {
            // HACK
            if self.check_sel4_test_kind_hack1("__sel4_root_task__main") {
                Some(SeL4TestKind::RootTask)
            } else if self.check_sel4_test_kind_hack1("__sel4_microkit__main") {
                Some(SeL4TestKind::Microkit)
            } else if self.check_sel4_test_kind_hack2("seL4_DebugPutChar") {
                Some(SeL4TestKind::RootTask)
            } else {
                None
            }
        }
    }

    fn check_sel4_test_kind(&self, symbol: &str) -> bool {
        self.file.symbol_by_name(symbol).is_some()
    }

    fn check_sel4_test_kind_hack1(&self, symbol: &str) -> bool {
        // impl happens to be same as above
        self.file.symbol_by_name(symbol).is_some()
    }

    fn check_sel4_test_kind_hack2(&self, suffix: &str) -> bool {
        self.file
            .symbols()
            .any(|symbol| symbol.name().unwrap().ends_with(suffix))
    }

    fn run_not_sel4(&self) -> anyhow::Result<()> {
        ensure!(
            Command::new(self.get_qemu_exe())
                .args(
                    iter::once(self.exe.as_os_str())
                        .chain(self.cli.simulate_args.iter().map(AsRef::as_ref)),
                )
                .status()?
                .success()
        );
        Ok(())
    }

    fn get_qemu_exe(&self) -> String {
        let qemu_arch = match self.file.architecture() {
            Architecture::Aarch64 => "aarch64",
            Architecture::Arm => "arm",
            Architecture::X86_64 => "x86_64",
            Architecture::X86_64_X32 => "i386",
            Architecture::Riscv32 => "riscv32",
            Architecture::Riscv64 => "riscv64",
            _ => unimplemented!(),
        };
        format!("qemu-{qemu_arch}")
    }

    fn is_resettable(&self) -> bool {
        self.file.symbol_by_name("_reset").is_some()
    }

    fn mk_resettable(&self) -> anyhow::Result<()> {
        if self.is_resettable() {
            let orig = self.exe.with_extension("orig.elf");
            fs::rename(self.exe, &orig)?;
            ensure!(
                Command::new("cargo")
                    .arg("run")
                    .arg("-p")
                    .arg("sel4-reset-cli")
                    .arg("--")
                    .arg(&orig)
                    .arg("-o")
                    .arg(self.exe)
                    .status()?
                    .success()
            );
            let sup = self.exe.with_extension("sup.elf");
            ensure!(
                Command::new("llvm-objcopy")
                    .arg("--only-keep-debug")
                    .arg(&orig)
                    .arg(&sup)
                    .status()?
                    .success()
            );
        }
        Ok(())
    }

    fn get_kernel_loader_target_config(&self) -> String {
        let target = match self.file.architecture() {
            Architecture::Aarch64 => "aarch64-unknown-none",
            Architecture::Arm => "armv7a-none-eabi",
            Architecture::X86_64 => "x86_64",
            Architecture::Riscv32 => "riscv32imac-unknown-none-elf", // TODO imac?
            Architecture::Riscv64 => "riscv64imac-unknown-none-elf", // TODO imac?
            _ => unimplemented!(),
        };
        format!(".cargo/gen/target/{target}.toml")
    }

    fn get_capdl_initializer_target_config(&self) -> String {
        let arch = match self.file.architecture() {
            Architecture::Aarch64 => "aarch64",
            Architecture::Arm => "armv7a",
            Architecture::X86_64 => "x86_64",
            Architecture::Riscv32 => "riscv32imac", // TODO imac?
            Architecture::Riscv64 => "riscv64imac", // TODO imac?
            _ => unimplemented!(),
        };
        let target = format!("{arch}-sel4-roottask-minimal");
        format!(".cargo/gen/target/{target}.toml")
    }

    fn mk_root_task_image(&self, root_task: &Path) -> anyhow::Result<PathBuf> {
        Ok(if let Architecture::X86_64 = self.file.architecture() {
            root_task.to_owned()
        } else {
            let image = self.d.join("image.elf");

            ensure!(
                Command::new("cargo")
                    .arg("build")
                    .arg("--config")
                    .arg(self.get_kernel_loader_target_config())
                    .arg("--target-dir")
                    .arg(&self.cli.target_dir)
                    .arg("-p")
                    .arg("sel4-kernel-loader")
                    .arg("--artifact-dir")
                    .arg(self.d)
                    .status()?
                    .success()
            );

            ensure!(
                Command::new("cargo")
                    .arg("run")
                    .arg("-p")
                    .arg("sel4-kernel-loader-add-payload")
                    .arg("--")
                    .arg("--loader")
                    .arg(self.d.join("sel4-kernel-loader"))
                    .arg("--sel4-prefix")
                    .arg(env::var("SEL4_PREFIX").unwrap())
                    .arg("--app")
                    .arg(root_task)
                    .arg("-o")
                    .arg(&image)
                    .status()?
                    .success()
            );

            image
        })
    }

    fn mk_microkit_image(&self) -> anyhow::Result<PathBuf> {
        let system_xml = self.d.join("system.xml");
        if let Some(sec) = self.file.section_by_name(".sdf_xml") {
            fs::write(&system_xml, sec.data()?)?;
        } else if let Some(sec) = self.file.section_by_name(".sdf_script") {
            let system_py = self.d.join("system.py");
            fs::write(&system_py, sec.data()?)?;
            ensure!(
                Command::new("python3")
                    .arg(&system_py)
                    .arg("--board")
                    .arg(self.cli.microkit_board.as_ref().unwrap())
                    .arg("-o")
                    .arg(&system_xml)
                    .status()?
                    .success()
            );
        } else {
            panic!("missing sdf")
        }

        let image = self.d.join("image.elf");

        ensure!(
            Command::new(
                self.cli
                    .microkit_sdk
                    .as_ref()
                    .unwrap()
                    .join("bin")
                    .join("microkit")
            )
            .arg(&system_xml)
            .arg("--search-path")
            .arg(self.d)
            .arg("--board")
            .arg(self.cli.microkit_board.as_ref().unwrap())
            .arg("--config")
            .arg(self.cli.microkit_config.as_ref().unwrap())
            .arg("-o")
            .arg(&image)
            .arg("-r")
            .arg(self.d.join("report.txt"))
            .status()?
            .success()
        );

        Ok(image)
    }

    fn mk_capdl_image(&self) -> anyhow::Result<PathBuf> {
        let script_out_dir = self.d.join("cdl");
        let sec = self
            .file
            .section_by_name(".capdl_script")
            .expect("missing script");
        let system_py = self.d.join("system.py");
        fs::write(&system_py, sec.data()?)?;
        ensure!(
            Command::new("python3")
                .arg(&system_py)
                .arg("--search-dir")
                .arg(self.d)
                .arg("--kernel")
                .arg(self.cli.kernel.as_ref().unwrap())
                .arg("--object-sizes")
                .arg(&self.cli.object_sizes)
                .arg("-o")
                .arg(&script_out_dir)
                .status()?
                .success()
        );

        let json = self.d.join("cdl.json");

        ensure!(
            Command::new("parse-capDL")
                .arg("--object-sizes")
                .arg(&self.cli.object_sizes)
                .arg("--json")
                .arg(&json)
                .arg(script_out_dir.join("spec.cdl"))
                .status()?
                .success()
        );

        ensure!(
            Command::new("cargo")
                .arg("build")
                .arg("--config")
                .arg(self.get_capdl_initializer_target_config())
                .arg("--target-dir")
                .arg(&self.cli.target_dir)
                .arg("-p")
                .arg("sel4-capdl-initializer")
                .arg("--artifact-dir")
                .arg(self.d)
                .status()?
                .success()
        );

        let root_task = self.d.join("root-task.elf");

        ensure!(
            Command::new("cargo")
                .arg("run")
                .arg("-p")
                .arg("sel4-capdl-initializer-add-spec")
                .arg("--")
                .arg("-e")
                .arg(self.d.join("sel4-capdl-initializer.elf"))
                .arg("-f")
                .arg(&json)
                .arg("-d")
                .arg(script_out_dir.join("links"))
                .arg("--object-names-level=2")
                .arg("--no-embed-frames")
                .arg("--no-deflate")
                .arg("-o")
                .arg(&root_task)
                .status()?
                .success()
        );

        self.mk_root_task_image(&root_task)
    }
}
