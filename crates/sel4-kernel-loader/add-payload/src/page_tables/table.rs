//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::borrow::Borrow;
use std::sync::Arc;

use super::regions::{AbstractRegion, AbstractRegions};
use super::scheme::{LeafDescriptor, Level, RawDescriptor, Scheme};

#[derive(Debug)]
pub(crate) struct Table {
    pub(crate) entries: Vec<AbstractEntry>,
}

#[derive(Debug)]
pub(crate) enum AbstractEntry {
    Empty,
    Leaf(RawDescriptor),
    Branch(Box<Table>),
}

pub(crate) trait MkLeafFn: Fn(MkLeafArgs<'_>) -> RawDescriptor {}

impl<F: Fn(MkLeafArgs) -> RawDescriptor> MkLeafFn for F {}

pub(crate) struct MkLeafArgs<'a> {
    scheme: &'a Scheme,
    level: Level,
    vaddr: u64,
}

impl<'a> MkLeafArgs<'a> {
    pub(crate) fn scheme(&self) -> &'a Scheme {
        self.scheme
    }

    pub(crate) fn descriptor<T: LeafDescriptor>(
        &self,
        vaddr_to_paddr: impl FnOnce(u64) -> u64,
    ) -> T {
        self.scheme()
            .leaf_descriptor_from_level_paddr(self.level, (vaddr_to_paddr)(self.vaddr))
    }

    pub(crate) fn identity_descriptor<T: LeafDescriptor>(&self) -> T {
        self.descriptor(|vaddr| vaddr)
    }
}

pub(crate) struct RegionContent {
    mk_leaf: Box<dyn MkLeafFn>,
    min_leaf_level_opt: Option<Level>,
}

impl RegionContent {
    pub(crate) fn new(mk_leaf: impl MkLeafFn + 'static) -> Self {
        Self {
            mk_leaf: Box::new(mk_leaf),
            min_leaf_level_opt: None,
        }
    }

    pub(crate) fn new_with_min_leaf_level(
        min_leaf_level: Level,
        mk_leaf: impl MkLeafFn + 'static,
    ) -> Self {
        Self {
            mk_leaf: Box::new(mk_leaf),
            min_leaf_level_opt: Some(min_leaf_level),
        }
    }
}

impl Table {
    pub(crate) fn construct(
        scheme: &Scheme,
        regions: &AbstractRegions<Option<RegionContent>>,
    ) -> Self {
        assert_eq!(regions.bounds(), scheme.virt_bounds());
        Construction::new(scheme, regions.as_slice().iter()).construct()
    }
}

struct Construction<'a, U, V> {
    scheme: &'a Scheme,
    current: U,
    rest: V,
}

impl<'a, U, V> Construction<'a, U, V>
where
    U: Borrow<AbstractRegion<Arc<Option<RegionContent>>>>,
    V: Iterator<Item = U>,
{
    fn new(scheme: &'a Scheme, mut regions: V) -> Self {
        let region = regions.next().unwrap();
        Self {
            scheme,
            current: region,
            rest: regions,
        }
    }

    fn construct(mut self) -> Table {
        let table = self.construct_inner(0, 0);
        assert!(!self.advance());
        table
    }

    fn construct_inner(&mut self, level: Level, table_vaddr: u64) -> Table {
        assert!(level < self.scheme.num_levels());
        let num_entries = self.scheme.num_entries_in_table(level);
        let step = 1 << self.scheme.leaf_size_bits(level);
        Table {
            entries: (0..num_entries)
                .map(|i| {
                    let entry_vaddr = table_vaddr + i * step;
                    while self.current_end() == entry_vaddr {
                        assert!(self.advance())
                    }
                    assert!(self.current_end() > entry_vaddr);
                    if self.current_end() < entry_vaddr + step
                        || self.current_content().is_some_and(|content| {
                            level
                                < content
                                    .min_leaf_level_opt
                                    .unwrap_or_else(|| self.scheme.min_level_for_leaf())
                        })
                    {
                        AbstractEntry::Branch(Box::new(
                            self.construct_inner(level + 1, entry_vaddr),
                        ))
                    } else {
                        match self.current_content() {
                            None => AbstractEntry::Empty,
                            Some(region_content) => {
                                AbstractEntry::Leaf((region_content.mk_leaf)(MkLeafArgs {
                                    scheme: self.scheme,
                                    level,
                                    vaddr: entry_vaddr,
                                }))
                            }
                        }
                    }
                })
                .collect(),
        }
    }

    fn current(&self) -> &AbstractRegion<Arc<Option<RegionContent>>> {
        self.current.borrow()
    }

    fn current_end(&self) -> u64 {
        self.current().range.end
    }

    fn current_content(&self) -> Option<&RegionContent> {
        (*self.current().content).as_ref()
    }

    fn advance(&mut self) -> bool {
        self.rest
            .next()
            .map(|region| self.current = region)
            .is_some()
    }
}
