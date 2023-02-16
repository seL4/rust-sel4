#![no_std]

use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{AsBytes, FromBytes};

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
pub enum RequestTag {
    PutChar,
    GetChar,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct PutCharRequest {
    pub val: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
pub enum GetCharResponseTag {
    None,
    Some,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct GetCharSomeResponse {
    pub val: u8,
}
