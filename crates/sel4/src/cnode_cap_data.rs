//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{newtype_methods, sys, Word, WORD_SIZE};

/// Corresponds to `seL4_CNode_CapData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CNodeCapData(sys::seL4_CNode_CapData);

impl CNodeCapData {
    newtype_methods!(sys::seL4_CNode_CapData);

    pub fn new(guard: Word, guard_size: usize) -> Self {
        Self::from_inner(sys::seL4_CNode_CapData::new(
            guard,
            guard_size.try_into().unwrap(),
        ))
    }

    pub fn skip(num_bits: usize) -> Self {
        Self::new(0, num_bits)
    }

    pub fn skip_high_bits(cnode_size_bits: usize) -> Self {
        Self::skip(WORD_SIZE - cnode_size_bits)
    }

    pub fn into_word(self) -> Word {
        let arr = self.inner().0.inner();
        assert_eq!(arr.len(), 1); // TODO assert at compile time instead
        arr[0]
    }
}
