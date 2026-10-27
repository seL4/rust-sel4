//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{digit0, multispace1, not_line_ending, satisfy},
    combinator::{all_consuming, cut, map, map_res, opt, recognize, value},
    error::{VerboseError, convert_error},
    multi::{many0, many0_count, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

pub mod ast;

use ast::*;

type Result<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

pub fn parse(text: &str) -> File {
    let entities = match all_consuming(terminated(many0(entity), ws))(text) {
        Ok((_, entities)) => entities,
        Err(nom::Err::Error(err) | nom::Err::Failure(err)) => {
            panic!("{}", convert_error(text, err))
        }
        Err(nom::Err::Incomplete(_)) => unreachable!(),
    };
    File::from_entities(entities)
}

enum RawEntity {
    Base(Base),
    Block(Block),
    TaggedUnion(TaggedUnion),
}

impl File {
    fn from_entities(entities: Vec<RawEntity>) -> Self {
        let mut current_base = None;
        let mut blocks = vec![];
        let mut tagged_unions = vec![];

        for entity in entities {
            match entity {
                RawEntity::Base(base) => current_base = Some(base),
                RawEntity::Block(block) => blocks.push(Entity {
                    base: current_base.clone().unwrap(),
                    inner: block,
                }),
                RawEntity::TaggedUnion(tagged_union) => tagged_unions.push(Entity {
                    base: current_base.clone().unwrap(),
                    inner: tagged_union,
                }),
            }
        }

        Self {
            blocks,
            tagged_unions,
        }
    }
}

fn token<'a, O>(
    inner: impl Parser<&'a str, O, VerboseError<&'a str>>,
) -> impl FnMut(&'a str) -> Result<'a, O> {
    preceded(ws, inner)
}

fn ws(input: &str) -> Result<'_, ()> {
    value(
        (),
        many0_count(alt((
            multispace1,
            recognize(pair(tag("--"), not_line_ending)),
        ))),
    )(input)
}

fn keyword<'a>(kw: &'static str) -> impl FnMut(&'a str) -> Result<'a, &'a str> {
    token(tag(kw))
}

fn ident(input: &str) -> Result<'_, Ident> {
    token(map(
        recognize(pair(
            satisfy(|c| c.is_ascii_alphabetic()),
            take_while(|c: char| c.is_ascii_alphanumeric() || c == '_'),
        )),
        str::to_owned,
    ))(input)
}

fn intlit(input: &str) -> Result<'_, usize> {
    token(map_res(
        alt((
            tag("0"),
            recognize(pair(satisfy(|c| matches!(c, '1'..='9')), digit0)),
        )),
        str::parse,
    ))(input)
}

fn paren_list<'a, O>(
    item: impl FnMut(&'a str) -> Result<'a, O>,
) -> impl FnMut(&'a str) -> Result<'a, Vec<O>> {
    delimited(
        keyword("("),
        separated_list1(keyword(","), item),
        keyword(")"),
    )
}

fn entity(input: &str) -> Result<'_, RawEntity> {
    alt((
        map(base, RawEntity::Base),
        map(block, RawEntity::Block),
        map(tagged_union, RawEntity::TaggedUnion),
    ))(input)
}

fn base(input: &str) -> Result<'_, Base> {
    map(
        preceded(
            keyword("base"),
            cut(pair(
                intlit,
                opt(delimited(
                    keyword("("),
                    separated_pair(intlit, keyword(","), intlit),
                    keyword(")"),
                )),
            )),
        ),
        |(base, base_mask)| {
            let (base_bits, sign_extend) = match base_mask {
                Some((base_bits, sign_extend)) => (base_bits, sign_extend == 1),
                None => (base, false),
            };
            Base {
                base,
                base_bits,
                sign_extend,
            }
        },
    )(input)
}

fn block(input: &str) -> Result<'_, Block> {
    map(
        preceded(
            keyword("block"),
            cut(tuple((
                ident,
                opt(paren_list(ident)),
                delimited(keyword("{"), many0(segment), keyword("}")),
            ))),
        ),
        |(name, visible_order_spec, segments)| Block {
            name,
            visible_order_spec,
            segments,
        },
    )(input)
}

fn segment(input: &str) -> Result<'_, Segment> {
    let field = map(
        pair(
            alt((
                value(true, keyword("field_high")),
                value(false, keyword("field")),
            )),
            cut(ident),
        ),
        |(is_high, name)| Some(Field { name, is_high }),
    );
    let padding = value(None, keyword("padding"));
    map(
        pair(alt((field, padding)), cut(intlit)),
        |(field, width)| Segment { width, field },
    )(input)
}

fn tagged_union(input: &str) -> Result<'_, TaggedUnion> {
    map(
        preceded(
            keyword("tagged_union"),
            cut(tuple((
                ident,
                ident,
                opt(paren_list(ident)),
                preceded(keyword("{"), many0(class)),
                terminated(many0(union_tag), keyword("}")),
            ))),
        ),
        |(name, tag_name, tag_slices, classes, tags)| TaggedUnion {
            tag_slices: tag_slices.unwrap_or_else(|| vec![tag_name.clone()]),
            name,
            tag_name,
            classes,
            tags,
        },
    )(input)
}

fn class(input: &str) -> Result<'_, Class> {
    map(
        preceded(keyword("mask"), cut(pair(intlit, intlit))),
        |(width, mask)| Class { width, mask },
    )(input)
}

fn union_tag(input: &str) -> Result<'_, Tag> {
    map(
        preceded(
            keyword("tag"),
            cut(pair(
                ident,
                alt((map(intlit, |v| vec![v]), paren_list(intlit))),
            )),
        ),
        |(name, values)| Tag { name, values },
    )(input)
}
