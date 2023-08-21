#![no_std]

use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{AsBytes, FromBytes};

pub type Microseconds = u64;

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum RequestTag {
    Now,
    SetTimeout,
    ClearTimeout,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct SetTimeoutRequest {
    pub relative_micros: Microseconds,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct NowResponse {
    pub micros: Microseconds,
}
