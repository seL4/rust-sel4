//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use crate::MessageInfo;

const BASE_OUTPUT_NOTIFICATION_SLOT: usize = 10;
const BASE_ENDPOINT_SLOT: usize = BASE_OUTPUT_NOTIFICATION_SLOT + 64;
const BASE_IRQ_SLOT: usize = BASE_ENDPOINT_SLOT + 64;
const BASE_TCB_SLOT: usize = BASE_IRQ_SLOT + 64;

const MAX_CHANNELS: usize = 63;

/// A channel between this protection domain and another, identified by a channel index.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Channel {
    index: usize,
}

impl Channel {
    pub const fn new(index: usize) -> Self {
        assert!(index < MAX_CHANNELS);
        Self { index }
    }

    fn cap<T: sel4::CapType>(&self, base_slot: usize) -> sel4::Cap<T> {
        sel4::Cap::from_bits((base_slot + self.index) as sel4::CPtrBits)
    }

    #[doc(hidden)]
    pub fn notification(&self) -> sel4::cap::Notification {
        self.cap::<sel4::cap_type::Notification>(BASE_OUTPUT_NOTIFICATION_SLOT)
    }

    #[doc(hidden)]
    pub fn irq_handler(&self) -> sel4::cap::IrqHandler {
        self.cap::<sel4::cap_type::IrqHandler>(BASE_IRQ_SLOT)
    }

    #[doc(hidden)]
    pub fn endpoint(&self) -> sel4::cap::Endpoint {
        self.cap::<sel4::cap_type::Endpoint>(BASE_ENDPOINT_SLOT)
    }

    pub fn notify(&self) {
        self.notification().signal()
    }

    pub fn irq_ack(&self) -> Result<(), IrqAckError> {
        self.irq_handler()
            .irq_handler_ack()
            .map_err(IrqAckError::from_inner)
    }

    pub fn pp_call(&self, msg_info: MessageInfo) -> MessageInfo {
        MessageInfo::from_inner(self.endpoint().call(msg_info.into_inner()))
    }
}

/// Error type returned by [`Channel::irq_ack`].
#[derive(Debug, PartialEq, Eq)]
pub struct IrqAckError(sel4::Error);

impl IrqAckError {
    fn from_inner(inner: sel4::Error) -> Self {
        Self(inner)
    }

    fn inner(&self) -> &sel4::Error {
        &self.0
    }
}

impl fmt::Display for IrqAckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "irq ack error: {:?}", self.inner())
    }
}

/// A handle to a child protection domain, identified by a child protection domain index.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProtectionDomain {
    index: usize,
}

impl ProtectionDomain {
    pub const fn new(index: usize) -> Self {
        Self { index }
    }

    #[doc(hidden)]
    pub fn tcb(&self) -> sel4::cap::Tcb {
        sel4::Cap::from_bits((BASE_TCB_SLOT + self.index) as sel4::CPtrBits)
    }
}
