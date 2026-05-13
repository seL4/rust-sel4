//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;

use super::regions::{AbstractRegion, AbstractRegions, AbstractRegionsBuilder};
use super::scheme::Scheme;
use super::table::{MkLeafFn, RegionContent, Table};

pub(crate) type Region = AbstractRegion<Option<RegionContent>>;
pub(crate) type Regions = AbstractRegions<Option<RegionContent>>;
pub(crate) type RegionsBuilder = AbstractRegionsBuilder<Option<RegionContent>>;

impl RegionsBuilder {
    pub(crate) fn new(scheme: &Scheme) -> Self {
        Self::new_with_background(Region::invalid(scheme.virt_bounds()))
    }
}

impl Regions {
    pub(crate) fn construct_table(&self, scheme: &Scheme) -> Table {
        Table::construct(scheme, self)
    }
}

impl Region {
    pub(crate) fn valid(range: Range<u64>, mk_leaf: impl MkLeafFn + 'static) -> Self {
        Self {
            range,
            content: Some(RegionContent::new(mk_leaf)),
        }
    }

    pub(crate) fn invalid(range: Range<u64>) -> Self {
        Self {
            range,
            content: None,
        }
    }
}
