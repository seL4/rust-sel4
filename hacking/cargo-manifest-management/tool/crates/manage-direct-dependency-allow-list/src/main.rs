//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use cargo_metadata::{semver::VersionReq, Metadata, MetadataCommand};
use clap::{Parser, Subcommand};
use toml::{Table, Value};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    CheckWorkspace(CheckWorkspaceArgs),
}

#[derive(Debug, Parser)]
struct CheckWorkspaceArgs {
    #[arg(long)]
    manifest_path: PathBuf,

    #[arg(long)]
    allowlist: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct AllowList {
    allow: HashMap<String, HashMap<VersionReq, HashSet<String>>>,
}

impl AllowList {
    fn deserialize(table: &Table) -> Self {
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

fn main() {
    match Cli::parse().command {
        Command::CheckWorkspace(args) => {
            let allowlist = AllowList::deserialize(
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
    }
}
