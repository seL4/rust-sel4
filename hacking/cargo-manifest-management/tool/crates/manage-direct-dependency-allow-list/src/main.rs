//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use anyhow::bail;
use cargo_metadata::semver::{Version, VersionReq};
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand};
use toml_edit::{Document, Formatted, Item, Table, Value};

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

    #[arg(long = "check")]
    just_check: bool,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::CheckWorkspace(args) => {
            let doc = fs::read_to_string(&args.allowlist)
                .unwrap()
                .parse::<Document>()
                .unwrap_or_else(|err| panic!("{err}"));
            let view = AllowListCheckWorkspaceView::new(&doc);
            let metadata = MetadataCommand::new()
                .manifest_path(&args.manifest_path)
                .no_deps()
                .exec()
                .unwrap();
            view.check(&metadata);
        }
        Command::Update(args) => {
            let mut doc = fs::read_to_string(&args.allowlist)
                .unwrap()
                .parse::<Document>()
                .unwrap_or_else(|err| panic!("{err}"));
            let mut view = AllowListUpdateView::new(&mut doc);
            let out_of_date = view.update();
            if out_of_date {
                if args.just_check {
                    bail!("out of date");
                } else {
                    let doc_str = doc.to_string();
                    match &args.out {
                        None => {
                            println!("{doc_str}");
                        }
                        Some(path) => {
                            fs::write(path, doc_str).unwrap();
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
    allow: BTreeMap<String, HashMap<VersionReq, BTreeSet<String>>>,
}

impl AllowListCheckWorkspaceView {
    fn new(table: &Table) -> Self {
        let mut this = Self::default();
        for (source_key, v) in table["allow"].as_table().unwrap().iter() {
            if let Item::Value(Value::String(req_str)) = v {
                let req = VersionReq::parse(req_str.value()).unwrap();
                this.insert_version(source_key, req, source_key);
            } else if let Some(v) = v.as_table_like() {
                let req = VersionReq::parse(v.get("version").unwrap().as_str().unwrap()).unwrap();
                for package_name in v
                    .get("applies-to")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                {
                    this.insert_version(package_name, req.clone(), source_key);
                }
            } else {
                panic!()
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
                    && !self.check_one(&dep.name, &dep.req)
                {
                    panic!("{} {} not in allowlist", dep.name, dep.req);
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
    req_slot: &'a mut Formatted<String>,
    applies_to: Vec<String>,
    auto_update: bool,
}

impl<'a> AllowListUpdateView<'a> {
    fn new(table: &'a mut Table) -> Self {
        let mut this = Self::default();
        for (key, value) in table["allow"].as_table_mut().unwrap().iter_mut() {
            let entry = if let Item::Value(Value::String(req_slot)) = value {
                let req = VersionReq::parse(req_slot.value()).unwrap();
                AllowListUpdateViewEntry {
                    req,
                    req_slot,
                    applies_to: vec![key.to_owned()],
                    auto_update: true,
                }
            } else if let Some(v) = value.as_table_like_mut() {
                let applies_to = v
                    .get("applies-to")
                    .map(|packages| {
                        packages
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_str().unwrap().to_owned())
                            .collect()
                    })
                    .unwrap_or_else(|| vec![key.to_owned()]);
                let auto_update = v
                    .get("auto-update")
                    .map(|v| v.as_bool().unwrap())
                    .unwrap_or(true);
                let req_slot = match v.get_mut("version").unwrap() {
                    Item::Value(Value::String(req_slot)) => req_slot,
                    _ => panic!(),
                };
                let req = VersionReq::parse(req_slot.value()).unwrap();
                AllowListUpdateViewEntry {
                    req,
                    req_slot,
                    applies_to,
                    auto_update,
                }
            } else {
                panic!()
            };
            this.allow.insert(key.get().to_owned(), entry);
        }
        this
    }

    fn update(&mut self) -> bool {
        let mut out_of_date = false;
        for (k, v) in self.allow.iter_mut() {
            if v.auto_update {
                let max_stable_version = v.fetch_max_stable_version();
                if !v.req.matches(&max_stable_version) {
                    out_of_date = true;
                    eprintln!("{}: {} -> {}", k, v.req, max_stable_version);
                    *v.req_slot = Formatted::new(max_stable_version.to_string());
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
        .unwrap_or_else(|| panic!("crate '{crate_name}' does not have a max_stable_version"));
    Version::parse(ver).unwrap()
}
