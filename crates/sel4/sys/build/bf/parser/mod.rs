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
    combinator::{all_consuming, cut, map, map_res, recognize, value},
    error::{VerboseError, convert_error},
    multi::{many0, many0_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
};

pub mod ast;

use ast::*;

type Result<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

pub fn parse(text: &str) -> File {
    match all_consuming(terminated(file, ws))(text) {
        Ok((_, file)) => file,
        Err(nom::Err::Error(err) | nom::Err::Failure(err)) => {
            panic!("{}", convert_error(text, err))
        }
        Err(nom::Err::Incomplete(_)) => unreachable!(),
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

fn file(input: &str) -> Result<'_, File> {
    map(pair(base, many0(declaration)), |(base, declarations)| {
        File { base, declarations }
    })(input)
}

fn declaration(input: &str) -> Result<'_, Declaration> {
    alt((
        map(block, Declaration::Block),
        map(tagged_union, Declaration::TaggedUnion),
    ))(input)
}

fn base(input: &str) -> Result<'_, usize> {
    preceded(keyword("base"), cut(intlit))(input)
}

fn block(input: &str) -> Result<'_, Block> {
    map(
        preceded(
            keyword("block"),
            cut(pair(
                ident,
                delimited(keyword("{"), many0(segment), keyword("}")),
            )),
        ),
        |(name, segments)| Block { name, segments },
    )(input)
}

fn segment(input: &str) -> Result<'_, Segment> {
    let field = map(preceded(keyword("field"), cut(ident)), Some);
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
                delimited(keyword("{"), many0(union_tag), keyword("}")),
            ))),
        ),
        |(name, tag_name, tags)| TaggedUnion {
            name,
            tag_name,
            tags,
        },
    )(input)
}

fn union_tag(input: &str) -> Result<'_, Tag> {
    map(
        preceded(keyword("tag"), cut(pair(ident, intlit))),
        |(name, value)| Tag { name, value },
    )(input)
}
