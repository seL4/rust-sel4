#![no_std]
#![feature(allocator_api)]
#![feature(btreemap_alloc)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate alloc;

use alloc::alloc::Global;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::alloc::{Allocator, Layout};

type Offset = usize;
type Size = usize;
type Align = usize;

struct AbstractBounceBufferAllocator<A: Allocator + Clone = Global> {
    align: Align,
    holes_by_offset: BTreeMap<Offset, Size, A>,
    holes_by_size: BTreeMap<Size, Vec<Offset, A>, A>,
}

struct AbstractBlock {
    offset: Offset,
    size: Size,
}

struct AbstractSubBlock {
    offset: Offset,
    size: Size,
}

enum Error {}

impl AbstractBounceBufferAllocator {
    fn new(size: Size, align: Align) -> Self {
        Self::new_in(Global, size, align)
    }
}

impl<A: Allocator + Clone> AbstractBounceBufferAllocator<A> {
    fn new_in(alloc: A, size: Size, align: Align) -> Self {
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
            align,
            holes_by_offset,
            holes_by_size,
        }
    }

    fn allocate(&mut self, layout: Layout) -> Result<AbstractBlock, Error> {
        todo!()
    }

    fn deallocate(&mut self, sub_block: AbstractSubBlock) {
        todo!()
    }
}

struct BounceBufferAllocator<A: Allocator + Clone = Global> {
    concrete_basis: *mut [u8],
    abstrat_allocator: AbstractBounceBufferAllocator<A>,
}

// impl BounceBufferAllocator {
//     fn new(region: *mut [u8]) -> Self {
//         Self::new_in(Global, size, align)
//     }
// }

// impl<A: Allocator + Clone> AbstractBounceBufferAllocator<A> {
//     fn new_in(alloc: A, size: Size, align: Align) -> Self {
//         let offset = 0;
//         let mut holes_by_offset = BTreeMap::new_in(alloc.clone());
//         holes_by_offset.insert(offset, size);
//         let mut holes_by_size = BTreeMap::new_in(alloc.clone());
//         holes_by_size.insert(size, {
//             let mut v = Vec::new_in(alloc.clone());
//             v.push(offset);
//             v
//         });
//         Self {
//             align,
//             holes_by_offset,
//             holes_by_size,
//         }
//     }

//     fn allocate(&mut self, layout: Layout) -> Result<AbstractBlock, Error> {
//         todo!()
//     }

//     fn deallocate(&mut self, sub_block: AbstractSubBlock) {
//         todo!()
//     }
// }
