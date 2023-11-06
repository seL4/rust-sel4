//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use std::str;

use clap::Parser;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use similar::{ChangeTag, TextDiff};
use toml::Table as TomlTable;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    blueprint: PathBuf,

    #[arg(long)]
    just_check: bool,
}

fn main() {
    let args = Args::parse();
    let blueprint_file = File::open(&args.blueprint).unwrap();
    let blueprint: Blueprint = serde_json::from_reader(blueprint_file).unwrap();
    blueprint.execute(args.just_check);
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Blueprint {
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    #[serde(rename = "absolutePath")]
    pub absolute_path: PathBuf,
    #[serde(rename = "manifestValue")]
    pub manifest_value: TomlTable,
    #[serde(rename = "frontmatter")]
    pub frontmatter: Option<String>,
    #[serde(rename = "formatPolicyOverrides")]
    pub format_policy_overrides: Vec<JsonValue>,
    #[serde(rename = "justEnsureEquivalence")]
    pub just_ensure_equivalence: bool,
}

impl Blueprint {
    pub fn execute(&self, just_check: bool) {
        for entry in self.entries.iter() {
            entry.execute(just_check);
        }
    }
}

impl Entry {
    fn execute(&self, just_check: bool) {
        let manifest_path = self.absolute_path.join("Cargo.toml");
        let rendered = self.render(&format!("{}", self.manifest_value));
        let mut write = true;
        if manifest_path.is_file() {
            let existing = fs::read_to_string(&manifest_path).unwrap();
            if self.just_ensure_equivalence {
                let existing_rendered = self.render(&existing);
                if existing_rendered != rendered {
                    eprintln!(
                        "error: {} is out of date (note that this is a structural comparison):",
                        manifest_path.display()
                    );
                    eprintln!("{}", format_diff(&rendered, &existing_rendered));
                    die();
                }
            } else {
                if existing == rendered {
                    write = false;
                } else if just_check {
                    eprintln!("error: {} is out of date:", manifest_path.display());
                    eprintln!("{}", format_diff(&rendered, &existing));
                    die();
                }
            }
        } else if just_check || self.just_ensure_equivalence {
            eprintln!("error: {} does not exist", manifest_path.display());
            die();
        }
        if write {
            eprintln!("writing {}", manifest_path.display());
            fs::write(manifest_path, rendered).unwrap();
        }
    }

    fn render(&self, unformatted_toml: &str) -> String {
        let mut cmd = Command::new("toml-normalize");
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("--builtin-policy=cargo-manifest");
        for policy in self.format_policy_overrides.iter() {
            cmd.arg(format!("--inline-policy={}", policy));
        }
        let mut child = cmd.spawn().unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(unformatted_toml.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let mut s = String::new();
        if let Some(frontmatter) = self.frontmatter.as_ref() {
            s.push_str(frontmatter);
            s.push('\n');
        }
        s.push_str(str::from_utf8(&output.stdout).unwrap());
        s
    }
}

fn format_diff(a: &str, b: &str) -> String {
    let d = TextDiff::from_lines(a, b);
    let mut s = String::new();
    for change in d.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        s.push_str(&format!("{}{}", sign, change))
    }
    s
}

fn die() -> ! {
    process::exit(1)
}
