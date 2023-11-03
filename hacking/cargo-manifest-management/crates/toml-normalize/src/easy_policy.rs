//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::cmp::{Ordering, Reverse};

use toml_path_regex::{PathRegex, PathSegment};

use crate::Policy;

pub struct EasyPolicy {
    pub max_width: usize,
    pub indent_width: usize,
    pub rules: Vec<TableRule>,
}

impl EasyPolicy {
    fn matching_rules<'a>(
        &'a self,
        path: &'a [PathSegment],
    ) -> impl 'a + Iterator<Item = &TableRule> {
        self.rules
            .iter()
            .rev()
            .filter(|rule| rule.path_regex.is_match(path.iter()))
    }
}

pub struct TableRule {
    pub path_regex: PathRegex,
    pub never_inline: bool,
    pub sort: TableRuleOrdering,
}

#[derive(Default)]
pub struct TableRuleOrdering {
    pub front: Vec<String>,
    pub back: Vec<String>,
}

impl Default for EasyPolicy {
    fn default() -> Self {
        Self {
            max_width: 100,
            indent_width: 4,
            rules: Default::default(),
        }
    }
}

impl Default for TableRule {
    fn default() -> Self {
        Self {
            path_regex: PathRegex::new("!(.*)"),
            never_inline: Default::default(),
            sort: Default::default(),
        }
    }
}

impl Policy for EasyPolicy {
    fn max_width(&self) -> usize {
        self.max_width
    }

    fn indent_width(&self) -> usize {
        self.indent_width
    }

    fn never_inline_table(&self, path: &[PathSegment]) -> bool {
        self.matching_rules(path).any(|rule| rule.never_inline)
    }

    fn compare_keys(&self, path: &[PathSegment], a: &str, b: &str) -> Ordering {
        for rule in self.matching_rules(path) {
            let ordering = rule.sort.compare(a, b);
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        a.cmp(b)
    }
}

impl TableRuleOrdering {
    fn key<'a>(&self, a: &'a str) -> (Reverse<Option<Reverse<usize>>>, Option<usize>, &'a str) {
        (
            Reverse(self.front.iter().position(|s| s == &a).map(Reverse)),
            self.back.iter().position(|s| s == &a),
            a,
        )
    }

    fn compare(&self, a: &str, b: &str) -> Ordering {
        self.key(a).cmp(&self.key(b))
    }
}
