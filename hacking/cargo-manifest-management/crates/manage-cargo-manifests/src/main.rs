//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};
use std::path::PathBuf;
use std::process;
use std::str;

use clap::Parser;
use serde::Deserialize;
use similar::{ChangeTag, TextDiff};
use toml::Table as TomlTable;

use toml_normalize::{builtin_policies, Error as TomlNormalizeError, Formatter, Policy};

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
    pub format_policy_overrides: Vec<Policy>,
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
        let rendered = self.render(&self.manifest_value).unwrap_or_else(|err| {
            eprintln!(
                "error normalizing structured value for {}: {}",
                manifest_path.display(),
                err
            );
            die();
        });
        let mut write = true;
        if manifest_path.is_file() {
            let existing = fs::read_to_string(&manifest_path).unwrap();
            if self.just_ensure_equivalence {
                let existing_toml = toml::from_str(&existing).unwrap_or_else(|err| {
                    eprintln!("error parsing {} as TOML: {}", manifest_path.display(), err);
                    die();
                });
                let existing_rendered = self.render(&existing_toml).unwrap_or_else(|err| {
                    eprintln!("error normalizing {}: {}", manifest_path.display(), err);
                    die();
                });
                if existing_rendered != rendered {
                    eprintln!(
                        "error: {} is out of date (note that this is a structural comparison):",
                        manifest_path.display()
                    );
                    eprintln!("{}", format_diff(&rendered, &existing_rendered));
                    die();
                }
            } else if existing == rendered {
                write = false;
            } else if just_check {
                eprintln!("error: {} is out of date:", manifest_path.display());
                eprintln!("{}", format_diff(&rendered, &existing));
                die();
            }
        } else if just_check || self.just_ensure_equivalence {
            eprintln!("error: {} does not exist", manifest_path.display());
            die();
        }
        if write {
            assert!(!just_check);
            eprintln!("writing {}", manifest_path.display());
            fs::write(manifest_path, rendered).unwrap();
        }
    }

    fn render(&self, unformatted_toml: &TomlTable) -> Result<String, TomlNormalizeError> {
        let mut composite_policy = builtin_policies::cargo_manifest_policy();
        for policy in self.format_policy_overrides.iter() {
            composite_policy = composite_policy.override_with(policy);
        }
        let formatter = Formatter::new(composite_policy);
        let doc = formatter.format(unformatted_toml)?;
        let mut s = String::new();
        if let Some(frontmatter) = self.frontmatter.as_ref() {
            s.push_str(frontmatter);
            s.push('\n');
        }
        s.push_str(&doc.to_string());
        Ok(s)
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
