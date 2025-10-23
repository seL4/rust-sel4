//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

//! Fault types.

use crate::{IpcBuffer, MessageInfo, sys};

pub use crate::arch::fault::*;

impl Fault {
    pub fn new(ipc_buffer: &IpcBuffer, info: &MessageInfo) -> Self {
        Self::from_sys(sys::seL4_Fault::get_from_ipc_buffer(
            info.inner(),
            ipc_buffer.inner(),
        ))
    }
}
