use core::fmt;

#[cfg(feature = "deflate")]
use core::iter;

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        use alloc::string::String;
        use alloc::vec::Vec;
    }
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentBytes<'a> {
    pub bytes: &'a [u8],
}

impl<'a> fmt::Debug for FillEntryContentBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FillEntryContentBytes")
            .field("bytes", &"&[...]")
            .finish()
    }
}

#[cfg(feature = "deflate")]
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentDeflatedBytes<'a> {
    pub deflated_bytes: &'a [u8],
}

#[cfg(feature = "deflate")]
impl<'a> fmt::Debug for FillEntryContentDeflatedBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FillEntryContentDeflatedBytes")
            .field("deflated_bytes", &"&[...]")
            .finish()
    }
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentDigest {
    pub content_digest: Vec<u8>,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentFile {
    pub file: String,
    pub file_offset: usize,
}

// // //

pub trait AvailableFillEntryContent {
    // TODO(nspin) error handling
    fn copy_out(&self, dst: &mut [u8]);
}

impl<'a> AvailableFillEntryContent for FillEntryContentBytes<'a> {
    fn copy_out(&self, dst: &mut [u8]) {
        dst.copy_from_slice(self.bytes)
        // unsafe {
        //     core::intrinsics::volatile_copy_nonoverlapping_memory(
        //         dst.as_mut_ptr(),
        //         self.bytes.as_ptr(),
        //         dst.len(),
        //     )
        // }
    }
}

#[cfg(feature = "deflate")]
impl<'a> AvailableFillEntryContent for FillEntryContentDeflatedBytes<'a> {
    fn copy_out(&self, dst: &mut [u8]) {
        let n = miniz_oxide::inflate::decompress_slice_iter_to_slice(
            dst,
            iter::once(self.deflated_bytes),
            false, // zlib_header
            true,  // ignore_adler32
        )
        .unwrap();
        assert_eq!(n, dst.len())
    }
}
