//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::str;

use serde::Deserialize;
use toml::Table as UnformattedTable;

use toml_normalize::{AbstractPolicy, Formatter};

use crate::display_diff;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Plan {
    pub entries: BTreeMap<PathBuf, Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub manifest: UnformattedTable,
    pub frontmatter: Option<String>,
    pub just_ensure_equivalence: bool,
}

impl Plan {
    pub fn execute<P: AbstractPolicy>(&self, formatter: &Formatter<P>, just_check: bool) {
        for (path, entry) in self.entries.iter() {
            assert!(!entry.just_ensure_equivalence); // TODO unimplemented
            let rendered = entry.render(formatter);
            let mut write = true;
            if path.is_file() {
                let existing = fs::read(path).unwrap();
                let existing = str::from_utf8(&existing).unwrap();
                if existing == rendered {
                    write = false;
                } else if just_check {
                    eprintln!("error: {} is out of date:", path.display());
                    eprintln!("{}", display_diff(&rendered, existing));
                    process::exit(1);
                }
            } else if just_check {
                eprintln!("error: {} does not exist", path.display());
                process::exit(1);
            }
            if write {
                eprintln!("writing {}", path.display());
                fs::write(path, rendered).unwrap();
            }
        }
    }

    pub fn get_entry_by_package_name(&self, name: &str) -> Option<&Entry> {
        self.entries
            .values()
            .filter(|entry| entry.get_package_name() == Some(name))
            .next()
    }
}

impl Entry {
    pub fn get_package_name(&self) -> Option<&str> {
        Some(
            self.manifest
                .get("package")?
                .as_table()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
        )
    }

    fn render<P: AbstractPolicy>(&self, formatter: &Formatter<P>) -> String {
        let doc = formatter.format(&self.manifest).unwrap();
        let mut s = String::new();
        if let Some(frontmatter) = self.frontmatter.as_ref() {
            s.push_str(frontmatter);
            s.push('\n');
        }
        s.push_str(&doc.to_string());
        s
    }
}
