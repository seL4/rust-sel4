use std::borrow::Borrow;
use std::ops::Range;
use std::sync::Arc;

use proc_macro2::{Ident, TokenStream};

use crate::embed::EntryForEmbedding;
use crate::regions::{AbstractRegion, AbstractRegions};
use crate::table::{AbstractEntry, MkLeafFnParams, RegionContent, Table, PHYS_BOUNDS};

pub fn construct_and_embed_table(
    ident: Ident,
    regions: impl Iterator<Item = impl Borrow<Region>>,
) -> TokenStream {
    Table::construct(regions).embed(ident)
}

pub type Region = AbstractRegion<RegionContent<Option<BlockDescriptor>>>;
pub type Regions = AbstractRegions<RegionContent<Option<BlockDescriptor>>>;

impl Regions {
    pub fn new() -> Self {
        Self::new_with(Region::invalid(PHYS_BOUNDS))
    }

    pub fn construct_and_embed_table(&self, ident: Ident) -> TokenStream {
        construct_and_embed_table(ident, self.as_slice().iter())
    }
}

impl Region {
    pub fn valid(
        range: Range<u64>,
        mk_block_descriptor: impl 'static + Fn(MkLeafFnParams) -> BlockDescriptor,
    ) -> Self {
        Self::new_concrete(range, 1, move |params| Some(mk_block_descriptor(params)))
    }

    pub fn invalid(range: Range<u64>) -> Self {
        Self::new_concrete(range, 0, |_| None)
    }

    fn new_concrete(
        range: Range<u64>,
        min_level_for_leaf: u64,
        mk_leaf: impl 'static + Fn(MkLeafFnParams) -> Option<BlockDescriptor>,
    ) -> Self {
        Self {
            range,
            content: RegionContent {
                min_level_for_leaf,
                mk_leaf: Arc::new(Box::new(mk_leaf)),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockDescriptor(pub u64);

impl BlockDescriptor {
    pub fn set_access_flag(self, value: bool) -> Self {
        Self(self.0 | (u64::from(value) << 10))
    }

    pub fn set_attribute_index(self, index: u64) -> Self {
        assert_eq!(index & !0b111, 0);
        Self(self.0 | (index << 2))
    }

    pub fn set_shareability(self, shareability: u64) -> Self {
        assert_eq!(shareability & !0b11, 0);
        Self(self.0 | (shareability << 8))
    }
}

impl<'a> From<&'a AbstractEntry<Option<BlockDescriptor>>>
    for EntryForEmbedding<'a, Option<BlockDescriptor>>
{
    fn from(entry: &'a AbstractEntry<Option<BlockDescriptor>>) -> Self {
        match entry {
            AbstractEntry::Leaf(leaf) => Self {
                offset: match leaf {
                    Some(block_descriptor) => block_descriptor.0,
                    None => 0,
                },
                ptr: None,
            },
            AbstractEntry::Branch(branch) => Self {
                offset: 0b11,
                ptr: Some(branch),
            },
        }
    }
}

impl MkLeafFnParams {
    pub fn mk(&self, vaddr_to_paddr: impl FnOnce(u64) -> u64) -> BlockDescriptor {
        BlockDescriptor((vaddr_to_paddr)(self.vaddr) | if self.level == 3 { 0b11 } else { 0b01 })
    }

    pub fn mk_identity(&self) -> BlockDescriptor {
        self.mk(|vaddr| vaddr)
    }
}
