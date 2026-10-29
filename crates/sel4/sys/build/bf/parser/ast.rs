//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub type Ident = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub base: usize,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Block(Block),
    TaggedUnion(TaggedUnion),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub name: Ident,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub width: usize,
    pub field: Option<Ident>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedUnion {
    pub name: Ident,
    pub tag_name: Ident,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: Ident,
    pub value: usize,
}
