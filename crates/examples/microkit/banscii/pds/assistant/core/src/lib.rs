//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use ab_glyph::{point, Font, FontRef, Glyph, Point, PxScale, ScaleFont};
use num_traits::Float;

pub struct Draft {
    pub width: usize,
    pub height: usize,
    pub pixel_data: Vec<u8>,
}

impl Draft {
    // Derived from:
    // https://github.com/alexheretic/ab-glyph/blob/main/dev/examples/ascii.rs
    pub fn new(subject: &str) -> Self {
        let font_data = include_bytes!("../assets/fonts/rock-salt/RockSalt-Regular.ttf");
        let font = FontRef::try_from_slice(font_data).unwrap();

        // Desired font pixel height
        let height: f32 = 12.4; // to get 80 chars across (fits most terminals); adjust as desired
        let px_height = height.ceil() as usize;

        // 2x scale in x direction to counter the aspect ratio of monospace characters.
        let scale = PxScale {
            x: height * 2.0,
            y: height,
        };

        let scaled_font = font.into_scaled(scale);

        let mut glyphs = Vec::new();
        layout(&scaled_font, point(0.0, 0.0), subject, &mut glyphs);

        // Find the most visually pleasing width to display
        let px_width = glyphs
            .iter()
            .rev()
            .map(|g| g.position.x + scaled_font.h_advance(g.id))
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;

        // Rasterize to greyscale
        let mut pixel_data = vec![0; px_width * px_height];
        for g in glyphs {
            if let Some(og) = scaled_font.outline_glyph(g) {
                let bounds = og.px_bounds();
                og.draw(|x, y, v| {
                    let x = x as f32 + bounds.min.x;
                    let y = y as f32 + bounds.min.y;
                    // There's still a possibility that the glyph clips the boundaries of the bitmap
                    if x >= 0.0 && (x as usize) < px_width && y >= 0.0 && (y as usize) < px_height {
                        pixel_data[(x as usize) + (y as usize) * px_width] =
                            (v * 255.0 + 0.5) as u8;
                    }
                })
            }
        }

        Self {
            width: px_width,
            height: px_height,
            pixel_data,
        }
    }
}

pub fn layout<F, SF>(font: SF, position: Point, text: &str, target: &mut Vec<Glyph>)
where
    F: Font,
    SF: ScaleFont<F>,
{
    let mut caret = position + point(0.0, font.ascent());
    let mut last_glyph: Option<Glyph> = None;
    for c in text.chars() {
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;

        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id);

        target.push(glyph);
    }
}
