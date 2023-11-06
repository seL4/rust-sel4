//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::cmp::{Ordering, Reverse};

use toml_path_regex::{PathRegex, PathSegment};

use crate::AbstractPolicy;

const DEFAULT_MAX_WIDTH: usize = 100;
const DEFAULT_INDENT_WIDTH: usize = 4;

#[derive(Debug, Clone, Default)]
pub struct Policy {
    pub max_width: Option<usize>,
    pub indent_width: Option<usize>,
    pub table_rules: Vec<TableRule>,
}

#[derive(Debug, Clone)]
pub struct TableRule {
    pub path_regex: PathRegex,
    pub never_inline: Option<bool>,
    pub key_ordering: KeyOrdering,
}

#[derive(Debug, Clone, Default)]
pub struct KeyOrdering {
    pub front: Vec<String>,
    pub back: Vec<String>,
}

impl Default for TableRule {
    fn default() -> Self {
        Self {
            path_regex: PathRegex::new("!(.*)").unwrap(),
            never_inline: Default::default(),
            key_ordering: Default::default(),
        }
    }
}

impl AbstractPolicy for Policy {
    fn max_width(&self) -> usize {
        self.max_width.unwrap_or(DEFAULT_MAX_WIDTH)
    }

    fn indent_width(&self) -> usize {
        self.indent_width.unwrap_or(DEFAULT_INDENT_WIDTH)
    }

    fn never_inline_table(&self, path: &[PathSegment]) -> bool {
        self.matching_rules_in_order_of_precedence(path)
            .filter_map(|rule| rule.never_inline)
            .any(|x| x)
    }

    fn compare_keys(&self, path: &[PathSegment], a: &str, b: &str) -> Ordering {
        for rule in self.matching_rules_in_order_of_precedence(path) {
            let ordering = rule.key_ordering.compare(a, b);
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        a.cmp(b)
    }
}

impl Policy {
    pub fn override_with(&self, other: &Self) -> Self {
        Self {
            max_width: other.max_width,
            indent_width: other.indent_width,
            table_rules: {
                let mut this = vec![];
                this.extend(self.table_rules.iter().cloned());
                this.extend(other.table_rules.iter().cloned());
                this
            },
        }
    }

    fn matching_rules_in_order_of_precedence<'a>(
        &'a self,
        path: &'a [PathSegment],
    ) -> impl 'a + Iterator<Item = &TableRule> {
        self.table_rules
            .iter()
            .rev()
            .filter(|rule| rule.path_regex.is_match(path.iter()))
    }
}

impl KeyOrdering {
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
