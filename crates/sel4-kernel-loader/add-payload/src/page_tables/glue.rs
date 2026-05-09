//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;

use super::regions::{AbstractRegion, AbstractRegions, AbstractRegionsBuilder};
use super::scheme::Scheme;
use super::table::{MkLeafFn, RegionContent, Table};

pub type Region = AbstractRegion<Option<RegionContent>>;
pub type Regions = AbstractRegions<Option<RegionContent>>;
pub type RegionsBuilder = AbstractRegionsBuilder<Option<RegionContent>>;

impl RegionsBuilder {
    pub fn new(scheme: &Scheme) -> Self {
        Self::new_with_background(Region::invalid(scheme.virt_bounds()))
    }
}

impl Regions {
    pub fn construct_table(&self, scheme: &Scheme) -> Table {
        Table::construct(scheme, self)
    }
}

impl Region {
    pub fn valid(range: Range<u64>, mk_leaf: impl MkLeafFn + 'static) -> Self {
        Self {
            range,
            content: Some(RegionContent::new(mk_leaf)),
        }
    }

    pub fn invalid(range: Range<u64>) -> Self {
        Self {
            range,
            content: None,
        }
    }
}
