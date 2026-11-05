//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::BTreeSet;
use std::ops::Range;

use super::parser;

pub fn simplify(orig_file: &parser::File) -> File {
    File::simplify(orig_file)
}

#[derive(Debug)]
pub struct File {
    pub base: usize,
    pub blocks: Vec<Block>,
    pub tagged_unions: Vec<TaggedUnion>,
}

#[derive(Debug)]
pub struct Block {
    pub name: String,
    pub multiple: usize,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub offset: usize,
    pub width: usize,
}

#[derive(Debug)]
pub struct TaggedUnion {
    pub name: String,
    pub tag_name: String,
    pub tag_range: Range<usize>,
    pub multiple: usize,
    pub tags: Vec<Tag>,
}

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub value: usize,
    pub fields: Vec<Field>,
}

impl File {
    fn simplify(orig_file: &parser::File) -> Self {
        let base = orig_file.base;
        let mut orig_blocks = vec![];
        let mut orig_tagged_unions = vec![];
        for declaration in orig_file.declarations.iter() {
            match declaration {
                parser::Declaration::Block(block) => orig_blocks.push(block),
                parser::Declaration::TaggedUnion(tagged_union) => {
                    orig_tagged_unions.push(tagged_union)
                }
            }
        }
        let tagged_unions = orig_tagged_unions
            .iter()
            .map(|orig_tagged_union| {
                let tag_name = orig_tagged_union.tag_name.clone();
                let blocks_with_values = orig_tagged_union
                    .tags
                    .iter()
                    .map(|orig_tag| {
                        let orig_block = orig_blocks
                            .iter()
                            .find(|orig_block| orig_block.name == orig_tag.name)
                            .unwrap();
                        let block = Block::simplify(base, orig_block);
                        (block, orig_tag.value)
                    })
                    .collect::<Vec<(Block, usize)>>();
                let multiple = unified(
                    blocks_with_values
                        .iter()
                        .map(|(block, _value)| block.multiple),
                );
                let tag_range = unified(blocks_with_values.iter().map(|(block, _value)| {
                    let field = block
                        .fields
                        .iter()
                        .find(|field| field.name == tag_name)
                        .unwrap();
                    field.offset..field.offset + field.width
                }));
                let tags = blocks_with_values
                    .into_iter()
                    .map(|(block, value)| Tag {
                        name: block.name,
                        value,
                        fields: block.fields,
                    })
                    .collect();
                TaggedUnion {
                    name: orig_tagged_union.name.clone(),
                    tag_name,
                    tag_range,
                    multiple,
                    tags,
                }
            })
            .collect::<Vec<TaggedUnion>>();
        let blocks_to_exclude = tagged_unions
            .iter()
            .flat_map(|tagged_union| tagged_union.tags.iter().map(|tag| tag.name.to_owned()))
            .collect::<BTreeSet<String>>();
        let blocks = orig_blocks
            .iter()
            .filter(|block| !blocks_to_exclude.contains(&block.name))
            .map(|block| Block::simplify(base, block))
            .collect();
        Self {
            base,
            blocks,
            tagged_unions,
        }
    }
}

impl Block {
    fn simplify(base: usize, orig: &parser::Block) -> Self {
        let mut cur_offset = 0;
        let mut fields = vec![];
        for segment in orig.segments.iter().rev() {
            if let Some(name) = &segment.field {
                fields.push(Field {
                    name: name.clone(),
                    offset: cur_offset,
                    width: segment.width,
                })
            }
            cur_offset += segment.width;
        }
        fields.reverse(); // order matters for ::new(...) parameter list
        let multiple = cur_offset / base;
        Self {
            name: orig.name.clone(),
            multiple,
            fields,
        }
    }
}

fn unified<T: Eq>(mut it: impl Iterator<Item = T>) -> T {
    let first = it.next().unwrap();
    assert!(it.all(|subsequent| subsequent == first));
    first
}
