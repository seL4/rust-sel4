//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

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
}

fn main() {
    match Cli::parse().command {
        Command::CheckWorkspace(args) => {
            let allowlist = AllowList::new(
                &fs::read_to_string(&args.allowlist)
                    .unwrap()
                    .parse::<Table>()
                    .unwrap_or_else(|err| panic!("{err}")),
            );
            let metadata = MetadataCommand::new()
                .manifest_path(&args.manifest_path)
                .no_deps()
                .exec()
                .unwrap();
            allowlist.check(&metadata);
        }
        Command::Update(args) => {
            let mut table = fs::read_to_string(&args.allowlist)
                .unwrap()
                .parse::<Table>()
                .unwrap_or_else(|err| panic!("{err}"));
            let mut view = AllowListUpdateView::new(&mut table);
            view.update();
            let formatter = Formatter::new(allowlist_policy());
            let table_str = formatter.format(&table).unwrap().to_string();
            match &args.out {
                None => {
                    println!("{table_str}");
                }
                Some(out) => {
                    fs::write(out, table_str).unwrap();
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct AllowList {
    allow: HashMap<String, HashMap<VersionReq, HashSet<String>>>,
}

impl AllowList {
    fn new(table: &Table) -> Self {
        let mut this = Self::default();
        for (source_key, v) in table["allow"].as_table().unwrap().iter() {
            match v {
                Value::String(req_str) => {
                    let package_name = source_key;
                    let req = VersionReq::parse(req_str).unwrap();
                    this.insert_version(package_name, req, source_key);
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
}

impl<'a> AllowListUpdateViewEntry<'a> {
    fn new(key: &String, value: &'a mut Value) -> Self {
        match value {
            Value::String(req_slot) => Self::new_helper(req_slot, vec![key.to_owned()]),
            Value::Table(v) => {
                let applies_to = v["applies-to"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect();
                let req_slot = match &mut v["version"] {
                    Value::String(req_slot) => req_slot,
                    _ => panic!(),
                };
                Self::new_helper(req_slot, applies_to)
            }
            _ => {
                panic!();
            }
        }
    }

    fn new_helper(req_slot: &'a mut String, applies_to: Vec<String>) -> Self {
        Self {
            req: VersionReq::parse(req_slot).unwrap(),
            req_slot,
            applies_to,
        }
    }

    fn fetch_max_stable_version(&self) -> Version {
        let mut it = self
            .applies_to
            .iter()
            .map(|crate_name| fetch_max_stable_version(crate_name));
        let v = it.next().unwrap();
        for v_ in it {
            assert_eq!(v_, v);
        }
        v
    }

    fn update(&mut self) {
        let max_stable_version = self.fetch_max_stable_version();
        if !self.req.matches(&max_stable_version) {
            eprintln!(
                "{:?}: {} -> {}",
                self.applies_to, self.req, max_stable_version
            );
            *self.req_slot = max_stable_version.to_string();
        }
    }
}

impl<'a> AllowListUpdateView<'a> {
    fn new(table: &'a mut Table) -> Self {
        let mut this = Self::default();
        for (k, v) in table["allow"].as_table_mut().unwrap().iter_mut() {
            this.allow
                .insert(k.clone(), AllowListUpdateViewEntry::new(k, v));
        }
        this
    }

    fn update(&mut self) {
        for (_k, v) in self.allow.iter_mut() {
            v.update();
        }
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
