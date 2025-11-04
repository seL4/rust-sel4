//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use pest::{
    Parser, Span,
    error::Error,
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PathRegexParser;

#[derive(Debug, Clone)]
pub enum Expr<'i> {
    Epsilon,
    Not(Box<Self>),
    Star(Box<Self>),
    Plus(Box<Self>),
    Optional(Box<Self>),
    Repeat(Box<Self>, Repetition),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Concat(Box<Self>, Box<Self>),
    Dot,
    KeySymbol(Span<'i>),
    IndexSymbol(Vec<IndexRange>),
}

#[derive(Debug, Clone)]
pub struct Repetition {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct IndexRange {
    pub start: Option<usize>,
    pub end: Option<usize>,
}

pub fn parse(s: &str) -> Result<Expr<'_>, Error<Rule>> {
    let mut top_level_pairs = PathRegexParser::parse(Rule::path_regex, &s)?;
    assert_eq!(top_level_pairs.len(), 1);
    let path_regex_pair = top_level_pairs.next().unwrap();
    assert_eq!(path_regex_pair.as_rule(), Rule::path_regex);
    let expr_pair = path_regex_pair.into_inner().next().unwrap();
    Ok(*Expr::parse(expr_pair))
}

impl<'i> Expr<'i> {
    fn pratt() -> PrattParser<Rule> {
        PrattParser::new()
            .op(Op::infix(Rule::or, Assoc::Right))
            .op(Op::infix(Rule::and, Assoc::Right))
            .op(Op::infix(Rule::concat, Assoc::Right))
            .op(Op::postfix(Rule::star))
            .op(Op::postfix(Rule::plus))
            .op(Op::postfix(Rule::optional))
            .op(Op::postfix(Rule::repeat))
            .op(Op::prefix(Rule::not))
    }

    fn parse(pair: Pair<'i, Rule>) -> Box<Self> {
        Self::parse_inner(pair, &Self::pratt())
    }

    fn parse_inner(pair: Pair<'i, Rule>, pratt: &PrattParser<Rule>) -> Box<Self> {
        assert_eq!(pair.as_rule(), Rule::opt_expr);

        if let Some(expr_pair) = pair.into_inner().next() {
            assert_eq!(expr_pair.as_rule(), Rule::expr);
            pratt
                .map_primary(|primary| match primary.as_rule() {
                    Rule::dot => Box::new(Self::Dot),
                    Rule::key_symbol => Box::new(Self::KeySymbol(Self::parse_key_symbol(primary))),
                    Rule::index_symbol => {
                        Box::new(Self::IndexSymbol(Self::parse_index_symbol(primary)))
                    }
                    Rule::opt_expr => Self::parse_inner(primary, pratt),
                    _ => unreachable!(),
                })
                .map_prefix(|op, rhs| {
                    Box::new(match op.as_rule() {
                        Rule::not => Self::Not(rhs),
                        _ => unreachable!(),
                    })
                })
                .map_postfix(|lhs, op| {
                    Box::new(match op.as_rule() {
                        Rule::star => Self::Star(lhs),
                        Rule::plus => Self::Plus(lhs),
                        Rule::optional => Self::Optional(lhs),
                        Rule::repeat => Self::Repeat(lhs, Repetition::parse(op)),
                        _ => unreachable!(),
                    })
                })
                .map_infix(|lhs, op, rhs| {
                    Box::new(match op.as_rule() {
                        Rule::or => Self::Or(lhs, rhs),
                        Rule::and => Self::And(lhs, rhs),
                        Rule::concat => Self::Concat(lhs, rhs),
                        _ => unreachable!(),
                    })
                })
                .parse(expr_pair.into_inner())
        } else {
            Box::new(Self::Epsilon)
        }
    }

    fn parse_key_symbol(pair: Pair<Rule>) -> Span {
        assert_eq!(pair.as_rule(), Rule::key_symbol);
        let key_symbol_regex_pair = pair.into_inner().next().unwrap();
        assert_eq!(key_symbol_regex_pair.as_rule(), Rule::key_symbol_regex);
        key_symbol_regex_pair.as_span()
    }

    fn parse_index_symbol(pair: Pair<Rule>) -> Vec<IndexRange> {
        assert_eq!(pair.as_rule(), Rule::index_symbol);
        let opt_index_symbol_ranges_pair = pair.into_inner().next().unwrap();
        assert_eq!(
            opt_index_symbol_ranges_pair.as_rule(),
            Rule::opt_index_symbol_ranges
        );
        opt_index_symbol_ranges_pair
            .into_inner()
            .next()
            .map(|index_symbol_ranges_pair| {
                assert_eq!(
                    index_symbol_ranges_pair.as_rule(),
                    Rule::index_symbol_ranges
                );
                index_symbol_ranges_pair
                    .into_inner()
                    .map(|index_symbol_range_pair| {
                        assert_eq!(index_symbol_range_pair.as_rule(), Rule::index_symbol_range);
                        IndexRange::parse(index_symbol_range_pair)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![])
    }
}

impl Repetition {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::repeat);
        let repeat_inner_pair = pair.into_inner().next().unwrap();
        assert_eq!(repeat_inner_pair.as_rule(), Rule::repeat_inner);
        let pair = repeat_inner_pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::repeat_exactly => {
                let intlit_pair = pair.into_inner().next().unwrap();
                assert_eq!(intlit_pair.as_rule(), Rule::intlit);
                let n = intlit_pair.as_str().parse().unwrap();
                Self {
                    min: Some(n),
                    max: Some(n),
                }
            }
            Rule::repeat_inclusive_range => {
                assert_eq!(pair.as_rule(), Rule::repeat_inclusive_range);
                let mut pairs = pair.into_inner();
                let min = Self::parse_side(pairs.next().unwrap());
                let max = Self::parse_side(pairs.next().unwrap());
                Self { min, max }
            }
            _ => unreachable!(),
        }
    }

    fn parse_side(pair: Pair<Rule>) -> Option<usize> {
        parse_range_side(pair, Rule::repeat_inclusive_range_side)
    }
}

impl IndexRange {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::index_symbol_range);
        let mut pairs = pair.into_inner();
        let start = Self::parse_side(pairs.next().unwrap());
        let end = Self::parse_side(pairs.next().unwrap());
        Self { start, end }
    }

    fn parse_side(pair: Pair<Rule>) -> Option<usize> {
        parse_range_side(pair, Rule::index_symbol_range_side)
    }
}

fn parse_range_side(pair: Pair<Rule>, rule: Rule) -> Option<usize> {
    assert_eq!(pair.as_rule(), rule);
    pair.into_inner().next().map(|intlit_pair| {
        assert_eq!(intlit_pair.as_rule(), Rule::intlit);
        intlit_pair.as_str().parse().unwrap()
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let ss = &[
            "(.)",
            r#".*![""abc""](.|.&.)"#,
            "[-]",
            "[1-]",
            "[-1]",
            "[1-1]",
            ".{,}",
            ".{1}",
            ".{,1}",
            ".{1,}",
            ".{1,1}",
            "",
            "()",
            "()()",
            ".().",
        ];
        for s in ss {
            parse(s).unwrap();
        }
    }
}
