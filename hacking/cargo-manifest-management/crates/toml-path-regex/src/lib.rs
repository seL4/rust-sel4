//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;

use pest::error::{Error as PestError, ErrorVariant};
use regex::Regex;

mod generic_regex;
mod parse;
mod path;
mod path_segment_predicate;

use generic_regex::GenericRegex;
use parse::{parse, Expr, Rule};
use path_segment_predicate::PathSegmentPredicate;

pub use path::{Path, PathSegment};

pub type Error = PestError<Rule>;

#[derive(Clone)]
pub struct PathRegex {
    pattern: String,
    inner: GenericRegex<PathSegmentPredicate>,
}

impl PathRegex {
    pub fn new(path_regex: &str) -> Result<Self, Error> {
        let expr = parse(path_regex)?;
        Ok(Self {
            pattern: path_regex.to_owned(),
            inner: generic_regex_from_expr(&expr)?,
        })
    }

    pub fn is_match(&self, path: impl Iterator<Item = impl Borrow<PathSegment>>) -> bool {
        self.inner.is_match(path)
    }

    pub fn as_str(&self) -> &str {
        &self.pattern
    }
}

fn generic_regex_from_expr(expr: &Expr) -> Result<GenericRegex<PathSegmentPredicate>, Error> {
    Ok(match expr {
        Expr::Epsilon => GenericRegex::epsilon(),
        Expr::Not(r) => GenericRegex::complement(generic_regex_from_expr(r)?),
        Expr::Star(r) => GenericRegex::star(generic_regex_from_expr(r)?),
        Expr::Plus(r) => GenericRegex::plus(generic_regex_from_expr(r)?),
        Expr::Optional(r) => GenericRegex::optional(generic_regex_from_expr(r)?),
        Expr::Repeat(r, repetition) => {
            GenericRegex::repeat(generic_regex_from_expr(r)?, repetition.min, repetition.max)
        }
        Expr::And(rhs, lhs) => {
            GenericRegex::and(generic_regex_from_expr(rhs)?, generic_regex_from_expr(lhs)?)
        }
        Expr::Or(rhs, lhs) => {
            GenericRegex::or(generic_regex_from_expr(rhs)?, generic_regex_from_expr(lhs)?)
        }
        Expr::Concat(rhs, lhs) => {
            GenericRegex::then(generic_regex_from_expr(rhs)?, generic_regex_from_expr(lhs)?)
        }
        Expr::Dot => GenericRegex::symbol(PathSegmentPredicate::any()),
        Expr::KeySymbol(key_regex) => {
            let re = key_regex.as_str();
            let anchored_re = format!("^{}$", re); // TODO is this sound?
            let compiled_re = Regex::new(&anchored_re).map_err(|err| {
                Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("{}", err),
                    },
                    key_regex.clone(),
                )
            })?;
            GenericRegex::symbol(PathSegmentPredicate::from_key_regex(compiled_re))
        }
        Expr::IndexSymbol(index_ranges) => {
            GenericRegex::symbol(PathSegmentPredicate::from_index_ranges(index_ranges))
        }
    })
}

impl fmt::Display for PathRegex {
    /// Shows the original regular expression.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for PathRegex {
    /// Shows the original regular expression.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PathRegex").field(&self.as_str()).finish()
    }
}

impl FromStr for PathRegex {
    type Err = Error;

    /// Attempts to parse a string into a regular expression
    fn from_str(s: &str) -> Result<Self, Error> {
        Self::new(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        assert!(PathRegex::new(r#"["package"]"#)
            .unwrap()
            .is_match([PathSegment::Key("package".to_owned())].iter()));
        assert!(PathRegex::new(r#".*["(.*-)?dependencies"]."#)
            .unwrap()
            .is_match(
                [
                    PathSegment::Key("dependencies".to_owned()),
                    PathSegment::Key("foo".to_owned())
                ]
                .iter()
            ));
    }
}
