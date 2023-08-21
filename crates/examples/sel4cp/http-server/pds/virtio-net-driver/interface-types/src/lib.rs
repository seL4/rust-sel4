#![no_std]

use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{AsBytes, FromBytes};

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct MacAddress(pub [u8; 6]);

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum RequestTag {
    GetMacAddress,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct GetMacAddressResponse {
    pub mac_address: MacAddress,
}
