use std::collections::BTreeSet;
use std::ops::Range;

use sel4_bitfield_parser::ast;

pub fn simplify(orig: &ast::File) -> File {
    File::simplify(orig)
}

#[derive(Debug)]
pub struct File {
    pub blocks: Vec<Block>,
    pub tagged_unions: Vec<TaggedUnion>,
}

#[derive(Debug)]
pub struct Block {
    pub name: String,
    pub backing_type: BackingType,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackingType {
    pub base: usize,
    pub multiple: usize,
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
    pub backing_type: BackingType,
    pub tags: Vec<Tag>,
}

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub value: usize,
    pub fields: Vec<Field>,
}

impl File {
    fn simplify(orig_file: &ast::File) -> Self {
        let tagged_unions = orig_file
            .tagged_unions
            .iter()
            .map(|orig_tagged_union| {
                let tag_name = orig_tagged_union.inner.tag_name.clone();
                let base = simplify_base(&orig_tagged_union.base);
                assert_eq!(orig_tagged_union.inner.tag_slices.len(), 1);
                assert_eq!(orig_tagged_union.inner.tag_slices[0], tag_name);
                assert!(orig_tagged_union.inner.classes.is_empty());
                let blocks_with_values = orig_tagged_union
                    .inner
                    .tags
                    .iter()
                    .map(|orig_tag| {
                        assert_eq!(orig_tag.values.len(), 1);
                        let value = orig_tag.values[0];
                        let orig_block = orig_file
                            .blocks
                            .iter()
                            .find(|orig_block| orig_block.inner.name == orig_tag.name)
                            .unwrap();
                        let block = Block::simplify(orig_block);
                        (block, value)
                    })
                    .collect::<Vec<(Block, usize)>>();
                let backing_type = unified(
                    blocks_with_values
                        .iter()
                        .map(|(block, _value)| &block.backing_type),
                )
                .clone();
                assert_eq!(backing_type.base, base);
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
                    name: orig_tagged_union.inner.name.clone(),
                    tag_name,
                    tag_range,
                    backing_type,
                    tags,
                }
            })
            .collect::<Vec<TaggedUnion>>();
        let blocks_to_exclude = tagged_unions
            .iter()
            .flat_map(|tagged_union| tagged_union.tags.iter().map(|tag| tag.name.to_owned()))
            .collect::<BTreeSet<String>>();
        let blocks = orig_file
            .blocks
            .iter()
            .filter(|block| !blocks_to_exclude.contains(&block.inner.name))
            .map(Block::simplify)
            .collect();
        Self {
            blocks,
            tagged_unions,
        }
    }
}

impl Block {
    fn simplify(orig: &ast::Entity<ast::Block>) -> Self {
        let base = simplify_base(&orig.base);
        assert!(orig.inner.visible_order_spec.is_none());
        let mut cur_offset = 0;
        let mut fields = vec![];
        for segment in orig.inner.segments.iter().rev() {
            if let Some(field) = &segment.field {
                assert!(!field.is_high);
                fields.push(Field {
                    name: field.name.clone(),
                    offset: cur_offset,
                    width: segment.width,
                })
            }
            cur_offset += segment.width;
        }
        fields.reverse(); // order matters for ::new(...) parameter list
        let multiple = cur_offset / base;
        Self {
            name: orig.inner.name.clone(),
            backing_type: BackingType { base, multiple },
            fields,
        }
    }
}

fn simplify_base(base: &ast::Base) -> usize {
    assert_eq!(base.base_bits, base.base);
    assert!(!base.sign_extend);
    base.base
}

fn unified<T: Eq>(mut it: impl Iterator<Item = T>) -> T {
    let first = it.next().unwrap();
    assert!(it.all(|subsequent| subsequent == first));
    first
}
