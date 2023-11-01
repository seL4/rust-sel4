//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::{path_regex::PathRegex, EasyPolicy, Policy, TableRule, TableRuleOrdering};

pub fn cargo_manifest_policy() -> impl Policy {
    EasyPolicy {
        rules: vec![
            TableRule {
                path_regex: PathRegex::new(r#".{,1}|["target|profile"].{,2}|["bin"][-]."#),
                never_inline: true,
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new(""),
                sort: TableRuleOrdering {
                    front: vec![
                        "package".to_owned(),
                        "lib".to_owned(),
                        "bin".to_owned(),
                        "features".to_owned(),
                        "dependencies".to_owned(),
                        "dev-dependencies".to_owned(),
                        "build-dependencies".to_owned(),
                        "workspace".to_owned(),
                        "profile".to_owned(),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new(r#"["package"]"#),
                sort: TableRuleOrdering {
                    front: vec!["name".to_owned(), "version".to_owned()],
                    back: vec!["description".to_owned()],
                },
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new(r#".*["(.*-)?dependencies"]."#),
                sort: TableRuleOrdering {
                    front: vec![
                        "path".to_owned(),
                        "git".to_owned(),
                        "branch".to_owned(),
                        "tag".to_owned(),
                        "rev".to_owned(),
                        "version".to_owned(),
                        "registry".to_owned(),
                        "default-features".to_owned(),
                        "features".to_owned(),
                        "optional".to_owned(),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TableRule {
                path_regex: PathRegex::new(r#"["target"]."#),
                sort: TableRuleOrdering {
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
                path_regex: PathRegex::new(r#"["workspace"]"#),
                sort: TableRuleOrdering {
                    back: vec!["members".to_owned(), "exclude".to_owned()],
                    ..Default::default()
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
