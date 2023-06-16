#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use alloc::alloc::Global;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::alloc::{Allocator, Layout};

use crate::{Align, Offset, Size};

struct Basic<A: Allocator + Clone = Global> {
    holes_by_offset: BTreeMap<Offset, Size, A>,
    holes_by_size: BTreeMap<Size, Vec<Offset, A>, A>,
}

enum BasicAllocatorError {}

impl Basic {
    fn new(size: Size) -> Self {
        Self::new_in(Global, size)
    }
}

impl<A: Allocator + Clone> Basic<A> {
    fn new_in(alloc: A, size: Size) -> Self {
        let offset = 0;
        let mut holes_by_offset = BTreeMap::new_in(alloc.clone());
        holes_by_offset.insert(offset, size);
        let mut holes_by_size = BTreeMap::new_in(alloc.clone());
        holes_by_size.insert(size, {
            let mut v = Vec::new_in(alloc.clone());
            v.push(offset);
            v
        });
        Self {
            holes_by_offset,
            holes_by_size,
        }
    }
}
