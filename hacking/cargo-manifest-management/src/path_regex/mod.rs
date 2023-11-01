//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::borrow::Borrow;

mod generic_regex;
mod parse;
mod path_segment_predicate;

use generic_regex::GenericRegex;
use parse::{parse, Expr};
use path_segment_predicate::PathSegmentPredicate;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
struct Path {
    inner: Vec<PathSegment>,
}

impl Path {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[PathSegment] {
        self.inner.as_slice()
    }

    pub fn push(&mut self, path_segment: PathSegment) {
        self.inner.push(path_segment)
    }

    pub fn push_key(&mut self, key: String) {
        self.push(PathSegment::Key(key))
    }

    pub fn push_index(&mut self, index: usize) {
        self.push(PathSegment::Index(index))
    }

    pub fn pop(&mut self) -> Option<PathSegment> {
        self.inner.pop()
    }
}

pub struct PathRegex {
    source: String,
    inner: GenericRegex<PathSegmentPredicate>,
}

impl PathRegex {
    pub fn new(path_regex: &str) -> Self {
        let expr = parse(path_regex);
        Self {
            source: path_regex.to_owned(),
            inner: generic_regex_from_expr(&expr),
        }
    }

    pub fn is_match(&self, path: impl Iterator<Item = impl Borrow<PathSegment>>) -> bool {
        self.inner.is_match(path)
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

fn generic_regex_from_expr(expr: &Expr) -> GenericRegex<PathSegmentPredicate> {
    match expr {
        Expr::Epsilon => GenericRegex::epsilon(),
        Expr::Not(r) => GenericRegex::complement(generic_regex_from_expr(r)),
        Expr::Star(r) => GenericRegex::star(generic_regex_from_expr(r)),
        Expr::Plus(r) => GenericRegex::plus(generic_regex_from_expr(r)),
        Expr::Optional(r) => GenericRegex::optional(generic_regex_from_expr(r)),
        Expr::Repeat(r, repetition) => {
            GenericRegex::repeat(generic_regex_from_expr(r), repetition.min, repetition.max)
        }
        Expr::And(rhs, lhs) => {
            GenericRegex::and(generic_regex_from_expr(rhs), generic_regex_from_expr(lhs))
        }
        Expr::Or(rhs, lhs) => {
            GenericRegex::or(generic_regex_from_expr(rhs), generic_regex_from_expr(lhs))
        }
        Expr::Concat(rhs, lhs) => {
            GenericRegex::then(generic_regex_from_expr(rhs), generic_regex_from_expr(lhs))
        }
        Expr::Dot => GenericRegex::symbol(PathSegmentPredicate::any()),
        Expr::KeySymbol(key_regex) => {
            GenericRegex::symbol(PathSegmentPredicate::from_key_regex(key_regex))
        }
        Expr::IndexSymbol(index_ranges) => {
            GenericRegex::symbol(PathSegmentPredicate::from_index_ranges(index_ranges))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        assert!(PathRegex::new(r#"["package"]"#)
            .is_match([PathSegment::Key("package".to_owned())].iter()));
        assert!(PathRegex::new(r#".*["(.*-)?dependencies"]."#).is_match(
            [
                PathSegment::Key("dependencies".to_owned()),
                PathSegment::Key("foo".to_owned())
            ]
            .iter()
        ));
    }
}
