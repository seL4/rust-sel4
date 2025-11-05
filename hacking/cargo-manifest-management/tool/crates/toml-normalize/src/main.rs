//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{CommandFactory, FromArgMatches, Parser};

use toml_normalize::{Formatter, Policy, builtin_policies};

#[derive(Debug, Parser)]
struct Args {
    in_file: Option<PathBuf>,

    #[arg(short)]
    out_file: Option<PathBuf>,

    #[arg(long, value_name = "POLICY_FILE")]
    policy: Vec<PathBuf>,

    #[arg(long, value_name = "POLICY")]
    inline_policy: Vec<String>,

    #[arg(long)]
    builtin_policy: Vec<String>,
}

fn main() {
    let cmd = Args::command();
    let matches = cmd.get_matches();
    let args = Args::from_arg_matches(&matches).unwrap_or_else(|err| {
        let mut cmd = Args::command();
        err.format(&mut cmd).exit()
    });

    let mut in_read = args
        .in_file
        .as_ref()
        .map(|path| {
            Box::new(
                std::fs::File::open(path)
                    .unwrap_or_else(|err| panic!("error opening input file: {}", err)),
            ) as Box<dyn Read>
        })
        .unwrap_or_else(|| Box::new(std::io::stdin()) as Box<dyn Read>);

    let in_string = {
        let mut this = String::new();
        in_read
            .read_to_string(&mut this)
            .unwrap_or_else(|err| panic!("error reading input file: {}", err));
        this
    };

    let in_toml = toml::from_str::<toml::Value>(&in_string).unwrap_or_else(|err| {
        panic!("error parsing input file: {}", err);
    });

    let mut policies: Vec<(Policy, usize)> = vec![];

    for (path, index) in args.policy.iter().zip(
        matches
            .indices_of("policy")
            .map(Iterator::collect)
            .unwrap_or_else(Vec::new),
    ) {
        let s = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("error reading policy file {}: {}", path.display(), err));
        let policy = serde_json::from_str(&s).unwrap_or_else(|err| {
            panic!(
                "error deserializing policy file {}: {}",
                path.display(),
                err
            )
        });
        policies.push((policy, index));
    }

    for (s, index) in args.inline_policy.iter().zip(
        matches
            .indices_of("inline_policy")
            .map(Iterator::collect)
            .unwrap_or_else(Vec::new),
    ) {
        let policy = serde_json::from_str(s)
            .unwrap_or_else(|err| panic!("error deserializing policy {}: {:?}", s, err));
        policies.push((policy, index));
    }

    for (name, index) in args.builtin_policy.iter().zip(
        matches
            .indices_of("builtin_policy")
            .map(Iterator::collect)
            .unwrap_or_else(Vec::new),
    ) {
        let policy = match name.as_str() {
            "cargo-manifest" => builtin_policies::cargo_manifest_policy(),
            _ => panic!("unrecognized builtin policy: {}", name),
        };
        policies.push((policy, index));
    }

    policies.sort_by_key(|(_policy, index)| *index);

    let mut composite_policy = Policy::default();
    for (policy, _index) in policies.iter() {
        composite_policy = policy.override_with(&composite_policy);
    }

    let out_toml = Formatter::new(composite_policy)
        .format(
            in_toml
                .as_table()
                .unwrap_or_else(|| panic!("input TOML document is not a table")),
        )
        .unwrap_or_else(|err| panic!("error normalizing TOML: {}", err));

    let mut out_write = args
        .out_file
        .as_ref()
        .map(|path| {
            Box::new(
                std::fs::File::open(path)
                    .unwrap_or_else(|err| panic!("error opening output file: {}", err)),
            ) as Box<dyn Write>
        })
        .unwrap_or_else(|| Box::new(std::io::stdout()) as Box<dyn Write>);

    out_write
        .write_fmt(format_args!("{}", &out_toml))
        .and_then(|_| out_write.flush())
        .unwrap_or_else(|err| panic!("error writing to output file: {}", err));
}
