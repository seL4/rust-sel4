use core::alloc::Layout;

use crate::{AbstractBounceBufferAllocator, Offset, Size};

pub struct Bump {
    watermark: Offset,
    end: Offset,
}

impl Bump {
    pub fn new(size: Size) -> Self {
        Self {
            watermark: 0,
            end: size,
        }
    }
}

impl AbstractBounceBufferAllocator for Bump {
    type Error = ();

    fn allocate(&mut self, layout: Layout) -> Result<Offset, Self::Error> {
        let offset = self.watermark.next_multiple_of(layout.align());
        let new_watermark = offset + layout.size();
        assert!(new_watermark <= self.end);
        self.watermark = new_watermark;
        Ok(offset)
    }

    fn deallocate(&mut self, _offset: Offset) {}
}
