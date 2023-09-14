#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    PutChar { val: u8 },
    GetChar,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCharSomeResponse {
    pub val: Option<u8>,
}
