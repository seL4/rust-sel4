//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(used_with_arg)]

mod channel;
mod message;
mod symbols;

pub use channel::{Channel, IrqAckError};
pub use message::{
    get_mr, set_mr, with_msg_bytes, with_msg_bytes_mut, with_msg_regs, with_msg_regs_mut,
    MessageInfo, MessageLabel, MessageRegisterValue,
};
pub use symbols::{ipc_buffer_ptr, pd_is_passive, pd_name};

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4_immutable_cell::ImmutableCell;
}
