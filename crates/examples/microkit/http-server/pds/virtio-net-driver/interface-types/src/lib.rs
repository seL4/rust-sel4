#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MacAddress(pub [u8; 6]);

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    GetMacAddress,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMacAddressResponse {
    pub mac_address: MacAddress,
}
