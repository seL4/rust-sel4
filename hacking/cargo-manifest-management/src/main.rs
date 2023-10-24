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

mod cargo_policy;
mod diff;
mod format;
mod plan;

use cargo_policy::CargoPolicy;
use diff::diff;
use format::{format, PathSegment, Policy};
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
    plan.execute::<CargoPolicy>(args.just_check);
}

// for debugging:

fn test_format() {
    let json_value = serde_json::from_reader(io::stdin()).unwrap();
    let toml_doc = format::<CargoPolicy>(&json_value);
    print!("{}", toml_doc)
}
