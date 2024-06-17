//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO make fallible?
pub trait GetBlockLayout {
    fn get_block_size(&mut self) -> usize;

    fn get_num_blocks(&mut self) -> u64;
}
