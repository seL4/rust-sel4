//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use rangemap::inclusive_set::RangeInclusiveSet;
use regex::Regex;

use super::{generic_regex::Predicate, parse::IndexRange, PathSegment};

pub struct PathSegmentPredicate {
    inner: Inner,
}

enum Inner {
    Any,
    Key(Regex),
    Index(RangeInclusiveSet<usize>),
}

impl PathSegmentPredicate {
    fn new(inner: Inner) -> Self {
        Self { inner }
    }

    pub fn any() -> Self {
        Self::new(Inner::Any)
    }

    pub fn from_key_regex(re: Regex) -> Self {
        Self::new(Inner::Key(re))
    }

    pub fn from_index_ranges(ranges: &[IndexRange]) -> Self {
        let mut set = RangeInclusiveSet::new();
        for range in ranges {
            let start = range.start.unwrap_or(usize::MIN);
            let end = range.end.unwrap_or(usize::MAX);
            set.insert(start..=end);
        }
        Self::new(Inner::Index(set))
    }
}

impl Predicate<PathSegment> for PathSegmentPredicate {
    fn is_match(&self, path_segment: &PathSegment) -> bool {
        match (&self.inner, path_segment) {
            (Inner::Any, _) => true,
            (Inner::Key(re), PathSegment::Key(key)) => re.is_match(key),
            (Inner::Index(set), PathSegment::Index(index)) => set.contains(index),
            _ => false,
        }
    }
}
