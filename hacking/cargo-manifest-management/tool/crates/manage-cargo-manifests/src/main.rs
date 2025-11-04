//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::str;

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde::Deserialize;
use similar::TextDiff;
use toml::Table as TomlTable;

use toml_normalize::{Error as TomlNormalizeError, Formatter, Policy, builtin_policies};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    blueprint: PathBuf,

    #[arg(long)]
    just_check: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let blueprint_file = File::open(&args.blueprint).unwrap();
    let blueprint: Blueprint =
        serde_json::from_reader(blueprint_file).context("error deserializeing blueprint")?;
    blueprint.execute(args.just_check)
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Blueprint {
    pub entries: Vec<Entry>,
}

// TODO these rename attributes cause a mix of caml and snake case
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
    pub fn execute(&self, just_check: bool) -> Result<()> {
        for entry in self.entries.iter() {
            entry.execute(just_check)?;
        }
        Ok(())
    }
}

impl Entry {
    fn execute(&self, just_check: bool) -> Result<()> {
        let manifest_path = self.absolute_path.join("Cargo.toml");
        let rendered_new = self.render(&self.manifest_value).with_context(|| {
            format!(
                "Failed to normalize structured value for {}",
                manifest_path.display()
            )
        })?;
        let mut write = false;
        if manifest_path.is_file() {
            let orig = fs::read_to_string(&manifest_path).unwrap();
            if self.just_ensure_equivalence {
                let orig_table = toml::from_str(&orig).with_context(|| {
                    format!("Failed to parse {} as TOML", manifest_path.display())
                })?;
                let normalized_orig = self
                    .render(&orig_table)
                    .with_context(|| format!("Failed to normalize {}", manifest_path.display()))?;
                if normalized_orig != rendered_new {
                    bail!(
                        "{} is out of date (note that this is a structural comparison):\n{}",
                        manifest_path.display(),
                        TextDiff::from_lines(&rendered_new, &normalized_orig).unified_diff(),
                    );
                }
            } else if orig != rendered_new {
                if just_check {
                    bail!(
                        "{} is out of date:\n{}",
                        manifest_path.display(),
                        TextDiff::from_lines(&rendered_new, &orig).unified_diff(),
                    );
                } else {
                    write = true;
                }
            }
        } else if just_check || self.just_ensure_equivalence {
            bail!("{} does not exist", manifest_path.display());
        } else {
            write = true;
        }
        if write {
            assert!(!just_check);
            eprintln!("writing {}", manifest_path.display());
            fs::write(manifest_path, rendered_new).unwrap();
        }
        Ok(())
    }

    fn render(&self, unformatted_toml: &TomlTable) -> StdResult<String, TomlNormalizeError> {
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
