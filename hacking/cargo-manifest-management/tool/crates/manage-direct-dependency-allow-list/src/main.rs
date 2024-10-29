//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::bail;
use cargo_metadata::semver::{Version, VersionReq};
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand};
use toml::{Table, Value};

use toml_normalize::{Formatter, KeyOrdering, Policy, TableRule};
use toml_path_regex::PathRegex;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    CheckWorkspace(CheckWorkspaceArgs),
    Update(UpdateArgs),
}

#[derive(Debug, Parser)]
struct CheckWorkspaceArgs {
    #[arg(long)]
    allowlist: PathBuf,

    #[arg(long)]
    manifest_path: PathBuf,
}

#[derive(Debug, Parser)]
struct UpdateArgs {
    #[arg(long)]
    allowlist: PathBuf,

    #[arg(short = 'o')]
    out: Option<PathBuf>,

    #[arg(long)]
    dry_run: bool,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::CheckWorkspace(args) => {
            let table = fs::read_to_string(&args.allowlist)
                .unwrap()
                .parse::<Table>()
                .unwrap_or_else(|err| panic!("{err}"));
            let view = AllowListCheckWorkspaceView::new(&table);
            let metadata = MetadataCommand::new()
                .manifest_path(&args.manifest_path)
                .no_deps()
                .exec()
                .unwrap();
            view.check(&metadata);
        }
        Command::Update(args) => {
            let mut table = fs::read_to_string(&args.allowlist)
                .unwrap()
                .parse::<Table>()
                .unwrap_or_else(|err| panic!("{err}"));
            let mut view = AllowListUpdateView::new(&mut table);
            let out_of_date = view.update();
            if out_of_date {
                if args.dry_run {
                    bail!("out of date");
                } else {
                    let formatter = Formatter::new(allowlist_policy());
                    let table_str = formatter.format(&table).unwrap().to_string();
                    match &args.out {
                        None => {
                            println!("{table_str}");
                        }
                        Some(path) => {
                            fs::write(path, table_str).unwrap();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct AllowListCheckWorkspaceView {
    allow: HashMap<String, HashMap<VersionReq, HashSet<String>>>,
}

impl AllowListCheckWorkspaceView {
    fn new(table: &Table) -> Self {
        let mut this = Self::default();
        for (source_key, v) in table["allow"].as_table().unwrap().iter() {
            match v {
                Value::String(req_str) => {
                    let req = VersionReq::parse(req_str).unwrap();
                    this.insert_version(source_key, req, source_key);
                }
                Value::Table(v) => {
                    let req = VersionReq::parse(v["version"].as_str().unwrap()).unwrap();
                    for package_name in v["applies-to"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap())
                    {
                        this.insert_version(package_name, req.clone(), source_key);
                    }
                }
                _ => {
                    panic!();
                }
            }
        }
        this
    }

    fn insert_version(&mut self, package_name: &str, req: VersionReq, source_key: &str) {
        self.allow
            .entry(package_name.to_owned())
            .or_default()
            .entry(req)
            .or_default()
            .insert(source_key.to_owned());
    }

    fn check(&self, metadata: &Metadata) {
        for package in &metadata.workspace_packages() {
            for dep in &package.dependencies {
                if dep
                    .source
                    .as_ref()
                    .map(|source| source.starts_with("registry+"))
                    .unwrap_or(false)
                {
                    if !self.check_one(&dep.name, &dep.req) {
                        panic!("{} {} not in allowlist", dep.name, dep.req);
                    }
                }
            }
        }
    }

    fn check_one(&self, dep: &str, req: &VersionReq) -> bool {
        if let Some(allowed_reqs) = self.allow.get(dep) {
            allowed_reqs.contains_key(req)
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
struct AllowListUpdateView<'a> {
    allow: HashMap<String, AllowListUpdateViewEntry<'a>>,
}

#[derive(Debug)]
struct AllowListUpdateViewEntry<'a> {
    req: VersionReq,
    req_slot: &'a mut String,
    applies_to: Vec<String>,
    allow_out_of_date: bool,
}

impl<'a> AllowListUpdateView<'a> {
    fn new(table: &'a mut Table) -> Self {
        let mut this = Self::default();
        for (key, value) in table["allow"].as_table_mut().unwrap().iter_mut() {
            let entry = match value {
                Value::String(req_slot) => {
                    let req = VersionReq::parse(req_slot).unwrap();
                    AllowListUpdateViewEntry {
                        req,
                        req_slot,
                        applies_to: vec![key.to_owned()],
                        allow_out_of_date: false,
                    }
                }
                Value::Table(v) => {
                    let applies_to = v["applies-to"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect();
                    let allow_out_of_date = v
                        .get("allow_out_of_date")
                        .map(|v| v.as_bool().unwrap())
                        .unwrap_or(false);
                    let req_slot = match &mut v["version"] {
                        Value::String(req_slot) => req_slot,
                        _ => panic!(),
                    };
                    let req = VersionReq::parse(req_slot).unwrap();
                    AllowListUpdateViewEntry {
                        req,
                        req_slot,
                        applies_to,
                        allow_out_of_date,
                    }
                }
                _ => {
                    panic!();
                }
            };
            this.allow.insert(key.clone(), entry);
        }
        this
    }

    fn update(&mut self) -> bool {
        let mut out_of_date = false;
        for (k, v) in self.allow.iter_mut() {
            let max_stable_version = v.fetch_max_stable_version();
            if !v.req.matches(&max_stable_version) {
                if !v.allow_out_of_date {
                    out_of_date = true;
                    eprintln!("{}: {} -> {}", k, v.req, max_stable_version);
                    *v.req_slot = max_stable_version.to_string();
                }
            }
        }
        out_of_date
    }
}

impl<'a> AllowListUpdateViewEntry<'a> {
    fn fetch_max_stable_version(&self) -> Version {
        let mut it = self
            .applies_to
            .iter()
            .map(AsRef::as_ref)
            .map(fetch_max_stable_version);
        let v_first = it.next().unwrap();
        for v in it {
            assert_eq!(v, v_first);
        }
        v_first
    }
}

fn fetch_max_stable_version(crate_name: &str) -> Version {
    let resp = ureq::get(&format!("https://crates.io/api/v1/crates/{crate_name}"))
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    let val = serde_json::from_str::<serde_json::Value>(&resp).unwrap();
    let ver = val.as_object().unwrap()["crate"].as_object().unwrap()["max_stable_version"]
        .as_str()
        .unwrap();
    Version::parse(ver).unwrap()
}

pub fn allowlist_policy() -> Policy {
    Policy {
        max_width: Some(usize::MAX),
        table_rules: vec![
            TableRule {
                path_regex: PathRegex::new(
                    r#"
                        .*
                    "#,
                )
                .unwrap(),
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new(
                    r#"
                        ['allow']
                    "#,
                )
                .unwrap(),
                never_inline: Some(true),
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new(
                    r#"
                        ['allow'] .
                    "#,
                )
                .unwrap(),
                key_ordering: KeyOrdering {
                    front: vec![
                        "version".to_owned(),
                        "applies-to".to_owned(),
                        "old".to_owned(),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
