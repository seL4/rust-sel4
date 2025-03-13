//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use regex::Regex;

pub mod ast;

use ast::*;

#[derive(Parser)]
#[grammar = "build/bf/parser/grammar.pest"]
struct BitfieldParser;

pub fn parse(text: &str) -> File {
    let text = remove_comments(text);
    let result = BitfieldParser::parse(Rule::file, &text);
    let pair = result
        .unwrap_or_else(|err| panic!("{}", err))
        .next()
        .unwrap();
    File::parse(pair)
}

fn remove_comments(text: &str) -> String {
    let re = Regex::new(r"--[^\n]*\n").unwrap();
    re.replace_all(text, "").into_owned()
}

impl File {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::file);

        let mut current_base = None;
        let mut blocks = vec![];
        let mut tagged_unions = vec![];

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::entity => {
                    let pair = pair.into_inner().next().unwrap();
                    match pair.as_rule() {
                        Rule::base => current_base = Some(Base::parse(pair)),
                        Rule::block => {
                            assert!(current_base.is_some());
                            blocks.push(Entity {
                                base: current_base.as_ref().unwrap().clone(),
                                inner: Block::parse(pair),
                            })
                        }
                        Rule::tagged_union => {
                            assert!(current_base.is_some());
                            tagged_unions.push(Entity {
                                base: current_base.as_ref().unwrap().clone(),
                                inner: TaggedUnion::parse(pair),
                            })
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                Rule::EOI => {}
                _ => {
                    unreachable!()
                }
            }
        }

        Self {
            blocks,
            tagged_unions,
        }
    }
}

impl Base {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::base);
        let mut pairs = pair.into_inner();
        let base = pairs.next().unwrap().as_str().parse().unwrap();
        let opt_base_mask = pairs.next().unwrap();
        assert_eq!(opt_base_mask.as_rule(), Rule::opt_base_mask);
        let (base_bits, sign_extend) = match opt_base_mask.into_inner().next() {
            Some(base_mask) => {
                assert_eq!(base_mask.as_rule(), Rule::base_mask);
                let mut pairs = base_mask.into_inner();
                (
                    pairs.next().unwrap().as_str().parse().unwrap(),
                    pairs.next().unwrap().as_str().parse::<usize>().unwrap() == 1,
                )
            }
            None => (base, false),
        };
        Self {
            base,
            base_bits,
            sign_extend,
        }
    }
}

impl Block {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::block);
        let mut pairs = pair.into_inner();
        let name = pairs.next().unwrap().as_str().to_owned();
        let opt_visible_order_spec = pairs.next().unwrap();
        assert_eq!(
            opt_visible_order_spec.as_rule(),
            Rule::opt_visible_order_spec
        );
        let visible_order_spec =
            opt_visible_order_spec
                .into_inner()
                .next()
                .map(|visible_order_spec| {
                    assert_eq!(visible_order_spec.as_rule(), Rule::visible_order_spec);
                    visible_order_spec
                        .into_inner()
                        .map(|pair| pair.as_str().to_owned())
                        .collect::<Vec<Ident>>()
                });
        let segments_pair = pairs.next().unwrap();
        assert_eq!(segments_pair.as_rule(), Rule::segments);
        let segments = segments_pair.into_inner().map(Segment::parse).collect();
        Self {
            name,
            visible_order_spec,
            segments,
        }
    }
}

impl Segment {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::segment);
        let mut pairs = pair.into_inner();
        let field = {
            let pair = pairs.next().unwrap();
            match pair.as_rule() {
                Rule::segment_field => {
                    let mut pairs = pair.into_inner();
                    let is_high = match pairs.next().unwrap().as_rule() {
                        Rule::field_low => false,
                        Rule::field_high => false,
                        _ => unreachable!(),
                    };
                    let name = pairs.next().unwrap().as_str().to_owned();
                    Some(Field { name, is_high })
                }
                Rule::segment_padding => None,
                _ => {
                    unreachable!()
                }
            }
        };
        let width = pairs.next().unwrap().as_str().parse().unwrap();
        Self { width, field }
    }
}

impl TaggedUnion {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::tagged_union);
        let mut pairs = pair.into_inner();
        let name = pairs.next().unwrap().as_str().to_owned();
        let tag_name = pairs.next().unwrap().as_str().to_owned();
        let opt_tag_slices = pairs.next().unwrap();
        assert_eq!(opt_tag_slices.as_rule(), Rule::opt_tag_slices);
        let tag_slices = match opt_tag_slices.into_inner().next() {
            Some(tag_slices_pair) => {
                assert_eq!(tag_slices_pair.as_rule(), Rule::tag_slices);
                tag_slices_pair
                    .into_inner()
                    .map(|pair| pair.as_str().to_owned())
                    .collect::<Vec<Ident>>()
            }
            None => {
                vec![tag_name.clone()]
            }
        };
        let classes_pair = pairs.next().unwrap();
        assert_eq!(classes_pair.as_rule(), Rule::classes);
        let classes = classes_pair.into_inner().map(Class::parse).collect();
        let tags_pair = pairs.next().unwrap();
        assert_eq!(tags_pair.as_rule(), Rule::tags);
        let tags = tags_pair.into_inner().map(Tag::parse).collect();
        Self {
            name,
            tag_name,
            tag_slices,
            classes,
            tags,
        }
    }
}

impl Class {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::class);
        let mut pairs = pair.into_inner();
        let width = pairs.next().unwrap().as_str().parse().unwrap();
        let mask = pairs.next().unwrap().as_str().parse().unwrap();
        Self { width, mask }
    }
}

impl Tag {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::tag);
        let mut pairs = pair.into_inner();
        let name = pairs.next().unwrap().as_str().to_owned();
        let values = {
            let pair = pairs.next().unwrap().into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::tag_value_one => {
                    vec![pair.into_inner().next().unwrap().as_str().parse().unwrap()]
                }
                Rule::tag_value_many => pair
                    .into_inner()
                    .map(|pair| pair.as_str().parse().unwrap())
                    .collect(),
                _ => {
                    unreachable!()
                }
            }
        };
        Self { name, values }
    }
}
