//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use cargo::core::source::SourceId;
use cargo::util::hex::short_hash;
use cargo_util::registry::make_dep_path;

use clap::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("")
        .subcommand_required(true)
        .subcommand(
            Command::new("make-dep-path")
                .arg(Arg::new("dep_name").required(true))
                .arg(
                    Arg::new("prefix_only")
                        .long("prefix-only")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(Command::new("registry-short-name").arg(Arg::new("url").required(true)))
        .get_matches();

    match matches.subcommand() {
        Some(("make-dep-path", sub_matches)) => {
            let dep_name = sub_matches.get_one::<String>("dep_name").unwrap();
            let prefix_only = sub_matches.get_flag("prefix_only");
            let path = make_dep_path(dep_name, prefix_only);
            println!("{}", path);
        }
        Some(("registry-short-name", sub_matches)) => {
            let url = sub_matches.get_one::<String>("url").unwrap();
            let short_name = registry_short_name(&url);
            println!("{}", short_name);
        }
        _ => {
            unreachable!()
        }
    }
}

fn registry_short_name(url: &str) -> String {
    let id = SourceId::for_registry(&url.parse().unwrap()).unwrap();
    let hash = short_hash(&id);
    let ident = id.url().host_str().unwrap_or("").to_string();
    format!("{}-{}", ident, hash)
}
