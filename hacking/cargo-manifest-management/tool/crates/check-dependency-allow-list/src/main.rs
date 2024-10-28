//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use cargo_metadata::{semver::VersionReq, Metadata, MetadataCommand};
use clap::Parser;
use toml::{Table, Value};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    manifest_path: PathBuf,

    #[arg(long)]
    allowlist: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct AllowList {
    versions: HashMap<String, HashSet<VersionReq>>,
}

impl AllowList {
    fn deserialize(table: &Table) -> Self {
        let mut this = Self::default();
        for (package_name, v) in table["versions"].as_table().unwrap().iter() {
            match v {
                Value::String(req_str) => {
                    let req = VersionReq::parse(req_str).unwrap();
                    this.insert_version(package_name, req);
                }
                Value::Table(v) => {
                    let req = VersionReq::parse(v["version"].as_str().unwrap()).unwrap();
                    for package_name in v["applies-to"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap())
                    {
                        this.insert_version(package_name, req.clone());
                    }
                }
                _ => {
                    panic!();
                }
            }
        }
        this
    }

    fn insert_version(&mut self, package_name: &str, req: VersionReq) {
        self.versions
            .entry(package_name.to_owned())
            .or_default()
            .insert(req);
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
        if let Some(allowed_reqs) = self.versions.get(dep) {
            allowed_reqs.contains(req)
        } else {
            false
        }
    }
}

fn main() {
    let args = Args::parse();
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
