//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use core::mem;
use core::slice;

#[sel4_cfg(KERNEL_INVOCATION_REPORT_ERROR_IPC)]
use core::str::{self, Utf8Error};

use crate::{cap, newtype_methods, sel4_cfg, sys, AbsoluteCPtr, Word};

#[sel4_cfg(KERNEL_INVOCATION_REPORT_ERROR_IPC)]
use crate::const_helpers::u32_into_usize;

/// Corresponds to `seL4_IPCBuffer`.
#[derive(Default)]
#[repr(transparent)]
pub struct IpcBuffer(sys::seL4_IPCBuffer);

impl IpcBuffer {
    newtype_methods!(pub sys::seL4_IPCBuffer);

    pub fn msg_regs(&self) -> &[Word] {
        &self.inner().msg[..]
    }

    pub fn msg_regs_mut(&mut self) -> &mut [Word] {
        &mut self.inner_mut().msg[..]
    }

    pub fn msg_bytes(&self) -> &[u8] {
        let msg = &self.inner().msg;
        let msg_ptr = msg as *const Word;
        let size = mem::size_of_val(msg);
        unsafe { slice::from_raw_parts(msg_ptr.cast(), size) }
    }

    pub fn msg_bytes_mut(&mut self) -> &mut [u8] {
        let msg = &mut self.inner_mut().msg;
        let msg_ptr = msg as *mut Word;
        let size = mem::size_of_val(msg);
        unsafe { slice::from_raw_parts_mut(msg_ptr.cast(), size) }
    }

    pub fn user_data(&self) -> Word {
        self.inner().userData
    }

    pub fn set_user_data(&mut self, data: Word) {
        self.inner_mut().userData = data;
    }

    pub fn caps_or_badges(&self) -> &[Word] {
        &self.inner().caps_or_badges[..]
    }

    pub fn caps_or_badges_mut(&mut self) -> &mut [Word] {
        &mut self.inner_mut().caps_or_badges[..]
    }

    pub fn recv_slot(&self) -> AbsoluteCPtr {
        let inner = self.inner();
        cap::CNode::from_bits(inner.receiveCNode).absolute_cptr_from_bits_with_depth(
            inner.receiveIndex,
            inner.receiveCNode.try_into().unwrap(),
        )
    }

    pub fn set_recv_slot(&mut self, slot: &AbsoluteCPtr) {
        let inner = self.inner_mut();
        inner.receiveCNode = slot.root().bits();
        inner.receiveIndex = slot.path().bits();
        inner.receiveDepth = slot.path().depth().try_into().unwrap();
    }

    #[sel4_cfg(KERNEL_INVOCATION_REPORT_ERROR_IPC)]
    pub fn debug_error_bytes(&self) -> &[u8] {
        let start = u32_into_usize(sys::DEBUG_MESSAGE_START) * mem::size_of::<Word>();
        let len = u32_into_usize(sys::DEBUG_MESSAGE_MAXLEN) * mem::size_of::<Word>();
        let all_bytes = &self.msg_bytes()[start..][..len];
        let n = all_bytes.iter().take_while(|b| **b != 0).count();
        &all_bytes[..n]
    }

    #[sel4_cfg(KERNEL_INVOCATION_REPORT_ERROR_IPC)]
    pub fn debug_error(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self.debug_error_bytes())
    }
}
