#![no_std]

use zerocopy::{AsBytes, FromBytes};

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct Request {
    pub height: usize,
    pub width: usize,
    pub draft_start: usize,
    pub draft_size: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct Response {
    pub height: usize,
    pub width: usize,
    pub masterpiece_start: usize,
    pub masterpiece_size: usize,
    pub signature_start: usize,
    pub signature_size: usize,
}
