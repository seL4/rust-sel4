//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(dead_code)]

use std::fs::File;
use std::io;
use std::path::PathBuf;

use clap::Parser;

mod cargo_manifest_policy;
mod diff;
mod format;
mod plan;

use cargo_manifest_policy::CargoManifestPolicy;
use diff::display_diff;
use format::{Formatter, PathSegment, Policy};
use plan::Plan;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    plan: PathBuf,

    #[arg(long)]
    just_check: bool,
}

fn main() {
    let args = Args::parse();
    let plan_file = File::open(&args.plan).unwrap();
    let plan: Plan = serde_json::from_reader(plan_file).unwrap();
    plan.execute(&Formatter::new(CargoManifestPolicy), args.just_check);
}

// for debugging:

fn test_format() {
    let root_table = serde_json::from_reader(io::stdin()).unwrap();
    let toml_doc = Formatter::new(CargoManifestPolicy)
        .format(&root_table)
        .unwrap();
    print!("{}", toml_doc)
}
