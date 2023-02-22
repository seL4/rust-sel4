#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use rusttype::{point, Font, Scale};

mod nostd_float;

use nostd_float::FloatExt;

pub struct Draft {
    pub width: usize,
    pub height: usize,
    pub pixel_data: Vec<u8>,
}

impl Draft {
    // Derived from:
    // https://github.com/redox-os/rusttype/blob/master/dev/examples/ascii.rs
    pub fn new(subject: &str) -> Self {
        let font_data = include_bytes!("../assets/fonts/rock-salt/RockSalt-Regular.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).unwrap();

        // Desired font pixel height
        let height: f32 = 12.4; // to get 80 chars across (fits most terminals); adjust as desired
        let pixel_height = height.ceil() as usize;

        // 2x scale in x direction to counter the aspect ratio of monospace characters.
        let scale = Scale {
            x: height * 2.0,
            y: height,
        };

        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        let glyphs = font.layout(subject, scale, offset).collect::<Vec<_>>();

        // Find the most visually pleasing width to display
        let width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;

        // Rasterise to greyscale
        let mut pixel_data = vec![0; width * pixel_height];
        for g in glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|x, y, v| {
                    let x = x as i32 + bb.min.x;
                    let y = y as i32 + bb.min.y;
                    // There's still a possibility that the glyph clips the boundaries of the bitmap
                    if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                        let x = x as usize;
                        let y = y as usize;
                        pixel_data[(x + y * width)] = (v * 255.0 + 0.5) as u8;
                    }
                })
            }
        }

        Self {
            width,
            height: pixel_height,
            pixel_data,
        }
    }
}
