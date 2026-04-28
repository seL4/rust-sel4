//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{Write, stdout};
use std::path::PathBuf;
use std::process::{Command, Output};

use cargo_metadata::{Metadata, MetadataCommand, PackageName};
use clap::Parser;
use regex::Regex;
use serde_json::{Value, json};

// HACK
fn project_root() -> PathBuf {
    let mut d = env::current_dir().unwrap();
    while !d.join("rust-toolchain.toml").exists() {
        d = d.parent().unwrap().to_owned();
    }
    d
}

fn main() {
    Env::get().run()
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, short = 'F')]
    features: Vec<String>,

    #[arg(long)]
    all_features: bool,

    #[arg(long)]
    no_default_features: bool,

    #[arg(long)]
    target: Option<String>,

    #[arg(long)]
    config: Vec<String>,

    #[arg(long, short = 'i')]
    include: Vec<String>,

    #[arg(long)]
    include_dependents: Vec<String>,

    #[arg(long, short = 'e')]
    exclude: Vec<String>,

    #[arg(long)]
    just_dump_includes: bool,

    #[arg(long)]
    just_dump_excludes: bool,

    #[arg(short = 'o')]
    out_path: Option<PathBuf>,
}

impl Cli {
    fn workspace_packages(&self) -> WorkspacePackages {
        let metadata = {
            let mut cmd = MetadataCommand::new();
            cmd.current_dir(project_root());
            cmd.other_options(self.forward_args());
            cmd.no_deps();
            cmd.exec().unwrap()
        };
        WorkspacePackages::from_metadata(&metadata)
    }

    fn forward_args(&self) -> Vec<String> {
        self.forward_args_with_feature_filter(|_| true)
    }

    fn forward_args_with_feature_filter(
        &self,
        feature_filter: impl Fn(&str) -> bool,
    ) -> Vec<String> {
        let mut args = self.forward_feature_args_with_feature_filter(&feature_filter);
        args.extend(self.forward_config_args());
        args
    }

    fn forward_feature_args(&self) -> Vec<String> {
        self.forward_feature_args_with_feature_filter(|_| true)
    }

    fn forward_feature_args_with_feature_filter(
        &self,
        feature_filter: impl Fn(&str) -> bool,
    ) -> Vec<String> {
        let mut args = vec![];
        for s in self.features.iter() {
            if let Some(filtered) = filter_features_arg(&feature_filter, s) {
                args.push("--features".to_owned());
                args.push(filtered.to_owned());
            }
        }
        if self.all_features {
            args.push("--all-features".to_owned());
        }
        if self.no_default_features {
            args.push("--no-default-features".to_owned());
        }
        args
    }

    fn forward_config_args(&self) -> Vec<String> {
        let mut args = vec![];
        if let Some(s) = self.target.as_ref() {
            args.push("--config".to_owned());
            args.push(format!("build.target=\"{s}\""));
        }
        for s in self.config.iter() {
            args.push("--config".to_owned());
            args.push(s.to_owned());
        }
        args
    }
}

fn filter_features_arg(feature_filter: impl Fn(&str) -> bool, arg: &str) -> Option<String> {
    let filtered = arg
        .split(',')
        .inspect(|s| check_feature_arg_element(s))
        .filter(|s| feature_filter(s))
        .collect::<Vec<_>>()
        .join(",");
    if filtered.is_empty() {
        None
    } else {
        Some(filtered)
    }
}

fn check_feature_arg_element(s: &str) {
    assert_eq!(s.chars().filter(|c| *c == '/').count(), 1)
}

struct Env {
    cli: Cli,
    ws: WorkspacePackages,
}

enum CargoTreeOutput<'a> {
    Packages(BTreeSet<&'a PackageName>),
    InvalidFeatures(Vec<String>),
}

impl<'a> CargoTreeOutput<'a> {
    fn parse(output: &Output, ws: &'a WorkspacePackages) -> Self {
        if output.status.success() {
            CargoTreeOutput::Packages(Self::parse_success(&output.stdout, ws))
        } else {
            CargoTreeOutput::InvalidFeatures(Self::parse_failure(&output.stderr))
        }
    }

    fn assume_success(output: &Output, ws: &'a WorkspacePackages) -> BTreeSet<&'a PackageName> {
        assert!(output.status.success());
        Self::parse_success(&output.stdout, ws)
    }

    fn parse_success(stdout: &[u8], ws: &'a WorkspacePackages) -> BTreeSet<&'a PackageName> {
        str::from_utf8(stdout)
            .unwrap()
            .lines()
            .filter_map(|s| {
                let r = r#"^(?<name>[a-zA-Z][a-zA-Z0-9_-]*) v[0-9.]+.* \(/[^)]+\)$"#;
                Regex::new(r)
                    .unwrap()
                    .captures(s)
                    .map(|captures| ws.by_name(captures.name("name").unwrap().as_str()))
            })
            .collect::<BTreeSet<_>>()
    }

    fn parse_failure(stderr: &[u8]) -> Vec<String> {
        let s = str::from_utf8(stderr).unwrap();
        let r = r#"error: the package '[a-zA-Z][a-zA-Z0-9_-]*' does not contain (this feature|these features): (?<feats>.+)"#;
        let feats = Regex::new(r)
            .unwrap()
            .captures(s)
            .unwrap()
            .name("feats")
            .unwrap()
            .as_str();
        feats.split(", ").map(|s| s.to_owned()).collect::<Vec<_>>()
    }
}

struct WorkspacePackages {
    pkgs: BTreeSet<PackageName>,
    pkgs_by_name: BTreeMap<String, PackageName>,
    root: PathBuf,
}

impl WorkspacePackages {
    fn from_metadata(metadata: &Metadata) -> Self {
        let pkgs = metadata
            .workspace_packages()
            .iter()
            .map(|pkg| pkg.name.clone())
            .collect::<BTreeSet<_>>();

        let mut pkgs_by_name = BTreeMap::new();
        for pkg in pkgs.iter() {
            pkgs_by_name.insert(pkg.to_string(), pkg.clone());
        }

        let root = metadata.workspace_root.clone().into();

        WorkspacePackages {
            pkgs,
            pkgs_by_name,
            root,
        }
    }

    fn iter(&self) -> impl Iterator<Item = &PackageName> {
        self.pkgs.iter()
    }

    fn by_name(&self, name: impl AsRef<str>) -> &PackageName {
        &self.pkgs_by_name[name.as_ref()]
    }

    fn by_names(&self, names: impl Iterator<Item = impl AsRef<str>>) -> BTreeSet<&PackageName> {
        names
            .map(|name| self.by_name(name))
            .collect::<BTreeSet<_>>()
    }
}

impl Env {
    fn get() -> Self {
        let cli = Cli::parse();
        let ws = cli.workspace_packages();
        Self { cli, ws }
    }

    fn include_dependents(&self) -> BTreeSet<&PackageName> {
        self.ws.by_names(self.cli.include_dependents.iter())
    }

    fn include_roots(&self) -> BTreeSet<&PackageName> {
        self.ws.by_names(self.cli.include.iter())
    }

    fn exclude_roots(&self) -> BTreeSet<&PackageName> {
        self.ws.by_names(self.cli.exclude.iter())
    }

    fn abs_out_path(&self) -> Option<PathBuf> {
        self.cli.out_path.as_ref().map(|p| {
            if p.is_absolute() {
                p.clone()
            } else {
                env::current_dir().unwrap().join(p)
            }
        })
    }

    fn create_out_file(&self) -> Box<dyn Write> {
        if let Some(p) = self.abs_out_path() {
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            Box::new(File::create(p).unwrap())
        } else {
            Box::new(stdout())
        }
    }

    fn run(&self) {
        let includes = self.get_included();
        let excludes = self.get_excluded(&includes);
        assert!(!(self.cli.just_dump_includes && self.cli.just_dump_excludes));
        if self.cli.just_dump_includes {
            let mut f = self.create_out_file();
            for pkg in includes.iter() {
                writeln!(f, "{pkg}").unwrap();
            }
        } else if self.cli.just_dump_excludes {
            let mut f = self.create_out_file();
            for pkg in excludes.iter() {
                writeln!(f, "{pkg}").unwrap();
            }
        } else {
            let vs_ws = self.vscode_workspace(excludes);
            let mut f = self.create_out_file();
            writeln!(f, "{vs_ws:#}").unwrap();
        }
    }

    fn vscode_workspace(&self, excludes: BTreeSet<&PackageName>) -> Value {
        let exclude_args = excludes
            .iter()
            .map(|x| format!("--exclude={x}"))
            .collect::<Vec<_>>()
            .join(" ");

        let mut new_settings = json!({
            "rust-analyzer.server.path": "hacking/vscode/rust-analyzer-wrapper",
            "rust-analyzer.linkedProjects": [
                "Cargo.toml",
            ],
            "rust-analyzer.cargo.allTargets": false,
            "rust-analyzer.cargo.extraArgs": self.cli.forward_config_args(),
            "rust-analyzer.cargo.metadataExtraArgs": self.cli.forward_config_args(),
            "rust-analyzer.cargo.extraEnv": {
                "__RUST_ANALYZER_WRAPPER__WORKSPACE_ARGS": exclude_args,
            },
            "terminal.integrated.env.linux": {
                "cargo_config_args": self.cli.forward_config_args().join(" "),
                "cargo_feature_args": self.cli.forward_feature_args().join(" "),
                "cargo_exclude_args": exclude_args,
            },
        });

        let new_settings_obj = new_settings.as_object_mut().unwrap();

        let features_val = if self.cli.all_features {
            if !self.cli.features.is_empty() {
                panic!()
            }
            Some(json!("all"))
        } else if !self.cli.features.is_empty() {
            Some(json!(self.cli.features))
        } else {
            None
        };
        if let Some(features_val) = features_val {
            new_settings_obj.insert("rust-analyzer.cargo.features".to_owned(), features_val);
        }
        if self.cli.no_default_features {
            new_settings_obj.insert(
                "rust-analyzer.cargo.noDefaultFeatures".to_owned(),
                json!(true),
            );
        }

        let mut settings = self.get_orig_settings();
        settings.as_object_mut().unwrap().append(new_settings_obj);

        let path = pathdiff::diff_paths(
            &self.ws.root,
            self.abs_out_path().unwrap().parent().unwrap(),
        )
        .unwrap();
        let path_str = path.to_str().unwrap();
        let path_str_for_json = if path_str.is_empty() { "." } else { path_str };

        json!({
            "folders": [
                { "path": path_str_for_json }
            ],
            "settings": settings,
        })
    }

    fn get_orig_settings(&self) -> Value {
        let bs = fs::read(self.ws.root.join(".vscode/settings.json")).unwrap();
        let s = str::from_utf8(&bs).unwrap();
        jsonc_parser::parse_to_serde_value(s, &Default::default()).unwrap()
    }

    fn get_included(&self) -> BTreeSet<&PackageName> {
        let include_roots = self.include_roots();
        let included_dependends = self.get_included_dependents();
        if include_roots.is_empty() && included_dependends.is_empty() {
            self.ws.iter().collect::<BTreeSet<_>>()
        } else {
            let output = {
                let mut cmd = self.cargo_tree_base_cmd();
                cmd.arg("--edges=no-build");
                cmd.arg("--edges=no-proc-macro");
                for pkg in self
                    .include_roots()
                    .iter()
                    .chain(self.get_included_dependents().iter())
                {
                    cmd.arg("--package").arg(pkg.as_str());
                }
                cmd
            }
            .output()
            .unwrap();
            CargoTreeOutput::assume_success(&output, &self.ws)
        }
    }

    fn get_included_dependents(&self) -> BTreeSet<&PackageName> {
        let include_dependents = self.include_dependents();
        if include_dependents.is_empty() {
            BTreeSet::new()
        } else {
            let output = {
                let mut cmd = self.cargo_tree_base_cmd();
                cmd.arg("--workspace");
                cmd.arg("--edges=no-build");
                cmd.arg("--edges=no-proc-macro");
                for pkg in include_dependents.iter() {
                    cmd.arg("--invert").arg(pkg.as_str());
                }
                cmd
            }
            .output()
            .unwrap();
            let mut pkgs = CargoTreeOutput::assume_success(&output, &self.ws);
            pkgs.retain(|possible_dependent| {
                self.has_dep_in(possible_dependent, &include_dependents)
            });
            pkgs
        }
    }

    fn get_excluded(&self, included: &BTreeSet<&PackageName>) -> BTreeSet<&PackageName> {
        let exclude_roots = self.exclude_roots();
        let fast_exclude_candidates = self.get_fast_exclude_candidates();
        self.ws
            .iter()
            .filter(|pkg| {
                !included.contains(*pkg)
                    || (fast_exclude_candidates.contains(*pkg)
                        && self.has_dep_in(pkg, &exclude_roots))
            })
            .collect::<BTreeSet<_>>()
    }

    fn get_fast_exclude_candidates(&self) -> BTreeSet<&PackageName> {
        let exclude_roots = self.exclude_roots();
        if exclude_roots.is_empty() {
            BTreeSet::new()
        } else {
            let output = {
                let mut cmd = self.cargo_tree_base_cmd();
                cmd.arg("--workspace");
                for pkg in exclude_roots.iter() {
                    cmd.arg("--invert").arg(pkg.as_str());
                }
                cmd
            }
            .output()
            .unwrap();
            CargoTreeOutput::assume_success(&output, &self.ws)
        }
    }

    fn has_dep_in(&self, pkg: &PackageName, deps_of_interest: &BTreeSet<&PackageName>) -> bool {
        !deps_of_interest.is_empty()
            && self
                .get_deps(pkg)
                .iter()
                .any(|dep| deps_of_interest.contains(dep))
    }

    fn get_deps(&self, pkg: &PackageName) -> BTreeSet<&PackageName> {
        match self.get_deps_inner(pkg, &[]) {
            CargoTreeOutput::Packages(pkgs) => pkgs,
            CargoTreeOutput::InvalidFeatures(feats) => match self.get_deps_inner(pkg, &feats) {
                CargoTreeOutput::Packages(pkgs) => pkgs,
                _ => panic!(),
            },
        }
    }

    fn get_deps_inner(
        &self,
        pkg: &PackageName,
        exclude_features: &[String],
    ) -> CargoTreeOutput<'_> {
        let output = {
            let mut cmd = self.cargo_tree_base_cmd();
            cmd.arg("--package").arg(pkg.as_ref());
            cmd.args(self.cli.forward_args_with_feature_filter(|feat| {
                !exclude_features
                    .iter()
                    .any(|excluded_feat| feat == excluded_feat)
            }));
            cmd
        }
        .output()
        .unwrap();
        CargoTreeOutput::parse(&output, &self.ws)
    }

    fn cargo_tree_base_cmd(&self) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.ws.root);
        cmd.args(["tree", "--prefix=none", "--format={p}", "--color=never"]);
        cmd
    }
}
