//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#[doc(hidden)]
#[macro_export]
macro_rules! declare_heap {
    ($size:expr) => {
        const _: () = {
            mod _scope {
                mod size_scope {
                    use super::super::*;
                    pub(super) const SIZE: usize = $size;
                }

                use $crate::_private::heap::*;

                static STATIC_HEAP: StaticHeap<{ size_scope::SIZE }> = StaticHeap::new();

                #[global_allocator]
                static GLOBAL_ALLOCATOR: StaticDlmalloc<RawOneShotMutex> =
                    StaticDlmalloc::new(STATIC_HEAP.bounds());
            }
        };
    };
}

pub mod _private {
    pub use one_shot_mutex::sync::RawOneShotMutex;
    pub use sel4_dlmalloc::{StaticDlmalloc, StaticHeap};
}
