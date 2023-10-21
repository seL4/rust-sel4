//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::vec::Vec;

pub(crate) struct Masterpiece {
    pub(crate) height: usize,
    pub(crate) width: usize,
    pub(crate) pixel_data: Vec<u8>,
}

impl Masterpiece {
    pub(crate) fn complete(
        draft_height: usize,
        draft_width: usize,
        draft_pixel_data: &[u8],
    ) -> Self {
        let height = draft_height;
        let width = draft_width;

        let mut pixel_data = draft_pixel_data.to_vec();

        for row in 0..height {
            for col in 0..width {
                let i = row * width + col;
                let grey = draft_pixel_data[i];
                pixel_data[i] = colorize(grey);
            }
        }

        Self {
            height,
            width,
            pixel_data,
        }
    }
}

const PALETTE: &[u8] = b"@%#x+=:-. ";

fn colorize(grey: u8) -> u8 {
    PALETTE
        [usize::from(grey) / (usize::from(u8::MAX).next_multiple_of(PALETTE.len()) / PALETTE.len())]
}
