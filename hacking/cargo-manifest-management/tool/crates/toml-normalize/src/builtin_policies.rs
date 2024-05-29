//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use toml_path_regex::PathRegex;

use crate::{KeyOrdering, Policy, TableRule};

pub fn cargo_manifest_policy() -> Policy {
    Policy {
        table_rules: vec![
            TableRule {
                path_regex: PathRegex::new(
                    r#"
                        .
                        | . [-]
                        | ['target'] .{2}
                        | ['profile'] . ['build-override']?
                        | ['package'] ['metadata'] .
                    "#,
                )
                .unwrap(),
                never_inline: Some(true),
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new("").unwrap(),
                key_ordering: KeyOrdering {
                    front: vec![
                        "package".to_owned(),
                        "lib".to_owned(),
                        "bin".to_owned(),
                        "example".to_owned(),
                        "test".to_owned(),
                        "bench".to_owned(),
                        "features".to_owned(),
                        "dependencies".to_owned(),
                        "dev-dependencies".to_owned(),
                        "build-dependencies".to_owned(),
                        "target".to_owned(),
                        "badges".to_owned(),
                        "features".to_owned(),
                        "patch".to_owned(),
                        "replace".to_owned(),
                        "profile".to_owned(),
                        "workspace".to_owned(),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new("['target'].").unwrap(),
                key_ordering: KeyOrdering {
                    front: vec![
                        "dependencies".to_owned(),
                        "dev-dependencies".to_owned(),
                        "build-dependencies".to_owned(),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new("['package']").unwrap(),
                key_ordering: KeyOrdering {
                    front: vec!["name".to_owned(), "version".to_owned()],
                    back: vec!["description".to_owned(), "metadata".to_owned()],
                },
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new(".*['(.*-)?dependencies'].").unwrap(),
                key_ordering: KeyOrdering {
                    front: vec![
                        "workspace".to_owned(),
                        "path".to_owned(),
                        "git".to_owned(),
                        "branch".to_owned(),
                        "tag".to_owned(),
                        "rev".to_owned(),
                        "version".to_owned(),
                        "registry".to_owned(),
                        "package".to_owned(),
                        "default-features".to_owned(),
                        "features".to_owned(),
                        "optional".to_owned(),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new("['workspace']").unwrap(),
                key_ordering: KeyOrdering {
                    back: vec!["members".to_owned(), "exclude".to_owned()],
                    ..Default::default()
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
