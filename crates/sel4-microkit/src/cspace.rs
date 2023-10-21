//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use crate::message::MessageInfo;

// For rustdoc.
#[allow(unused_imports)]
use crate::Handler;

pub(crate) type Slot = usize;

pub(crate) const INPUT_CAP: sel4::Endpoint = slot_to_local_cptr(1);
pub(crate) const REPLY_CAP: sel4::Reply = slot_to_local_cptr(4);
pub(crate) const MONITOR_EP_CAP: sel4::Endpoint = slot_to_local_cptr(5);

const BASE_OUTPUT_NOTIFICATION_CAP: Slot = 10;
const BASE_ENDPOINT_CAP: Slot = BASE_OUTPUT_NOTIFICATION_CAP + 64;
const BASE_IRQ_CAP: Slot = BASE_ENDPOINT_CAP + 64;

const MAX_CHANNELS: Slot = 63;

const fn slot_to_local_cptr<T: sel4::CapType>(slot: Slot) -> sel4::LocalCPtr<T> {
    sel4::LocalCPtr::from_bits(slot as sel4::CPtrBits)
}

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

    fn local_cptr<T: sel4::CapType>(&self, offset: Slot) -> sel4::LocalCPtr<T> {
        slot_to_local_cptr(offset + self.index)
    }

    fn notification(&self) -> sel4::Notification {
        self.local_cptr::<sel4::cap_type::Notification>(BASE_OUTPUT_NOTIFICATION_CAP)
    }

    fn irq_handler(&self) -> sel4::IRQHandler {
        self.local_cptr::<sel4::cap_type::IRQHandler>(BASE_IRQ_CAP)
    }

    fn endpoint(&self) -> sel4::Endpoint {
        self.local_cptr::<sel4::cap_type::Endpoint>(BASE_ENDPOINT_CAP)
    }

    pub fn notify(&self) {
        self.notification().signal()
    }

    pub fn irq_ack(&self) -> Result<(), IrqAckError> {
        self.irq_handler()
            .irq_handler_ack()
            .map_err(IrqAckError::from_sel4_error)
    }

    pub fn pp_call(&self, msg_info: MessageInfo) -> MessageInfo {
        MessageInfo::from_sel4(self.endpoint().call(msg_info.into_sel4()))
    }

    /// Prepare a [`DeferredAction`] for syscall coalescing using [`Handler::take_deferred_action`].
    pub fn defer_notify(&self) -> DeferredAction {
        DeferredAction::new(*self, DeferredActionInterface::Notify)
    }

    /// Prepare a [`DeferredAction`] for syscall coalescing using [`Handler::take_deferred_action`].
    pub fn defer_irq_ack(&self) -> DeferredAction {
        DeferredAction::new(*self, DeferredActionInterface::IrqAck)
    }
}

/// An action deferred for syscall coalescing using [`Handler::take_deferred_action`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DeferredAction {
    channel: Channel,
    interface: DeferredActionInterface,
}

/// A channel interface for which actions can be deferred.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DeferredActionInterface {
    Notify,
    IrqAck,
}

impl DeferredAction {
    pub fn new(channel: Channel, interface: DeferredActionInterface) -> Self {
        Self { channel, interface }
    }

    pub fn channel(&self) -> Channel {
        self.channel
    }

    pub fn interface(&self) -> DeferredActionInterface {
        self.interface
    }

    pub fn execute_now(self) -> Result<(), IrqAckError> {
        match self.interface() {
            DeferredActionInterface::Notify => {
                self.channel().notify();
                Ok(())
            }
            DeferredActionInterface::IrqAck => self.channel().irq_ack(),
        }
    }

    pub(crate) fn prepare(&self) -> PreparedDeferredAction {
        match self.interface() {
            DeferredActionInterface::Notify => PreparedDeferredAction::new(
                self.channel().notification().cast(),
                sel4::MessageInfoBuilder::default().build(),
            ),
            DeferredActionInterface::IrqAck => PreparedDeferredAction::new(
                self.channel().irq_handler().cast(),
                sel4::MessageInfoBuilder::default()
                    .label(sel4::sys::invocation_label::IRQAckIRQ.into())
                    .build(),
            ),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct PreparedDeferredAction {
    cptr: sel4::Unspecified,
    msg_info: sel4::MessageInfo,
}

impl PreparedDeferredAction {
    pub(crate) fn new(cptr: sel4::Unspecified, msg_info: sel4::MessageInfo) -> Self {
        Self { cptr, msg_info }
    }

    pub(crate) fn cptr(&self) -> sel4::Unspecified {
        self.cptr
    }

    pub(crate) fn msg_info(&self) -> sel4::MessageInfo {
        self.msg_info.clone() // TODO
    }
}

// TODO maybe excessive. remove?
pub struct DeferredActionSlot {
    inner: Option<DeferredAction>,
}

impl DeferredActionSlot {
    pub const fn new() -> Self {
        Self { inner: None }
    }

    pub fn take(&mut self) -> Option<DeferredAction> {
        self.inner.take()
    }

    pub fn defer(&mut self, action: DeferredAction) -> Result<(), IrqAckError> {
        self.inner
            .replace(action)
            .map(DeferredAction::execute_now)
            .unwrap_or(Ok(()))
    }
}

/// Error type returned by [`Channel::irq_ack`].
#[derive(Debug, PartialEq, Eq)]
pub struct IrqAckError {
    sel4_error: sel4::Error,
}

impl IrqAckError {
    fn from_sel4_error(sel4_error: sel4::Error) -> Self {
        Self { sel4_error }
    }

    fn as_sel4_error(&self) -> &sel4::Error {
        &self.sel4_error
    }
}

impl fmt::Display for IrqAckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "irq ack error: {:?}", self.as_sel4_error())
    }
}
