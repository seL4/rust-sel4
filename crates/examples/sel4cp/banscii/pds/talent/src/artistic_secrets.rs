use alloc::vec::Vec;

const PALETTE: &[u8] = b"@%#x+=:-. ";

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
                let color = PALETTE[usize::from(grey / 26)];
                pixel_data[i] = color;
            }
        }

        Self {
            height,
            width,
            pixel_data,
        }
    }
}
