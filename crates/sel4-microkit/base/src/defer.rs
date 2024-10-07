//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::{Channel, IrqAckError};

// For rustdoc
#[allow(unused_imports)]
use crate::Handler;

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

    pub fn new_notify(channel: Channel) -> DeferredAction {
        DeferredAction::new(channel, DeferredActionInterface::Notify)
    }

    pub fn new_irq_ack(channel: Channel) -> DeferredAction {
        DeferredAction::new(channel, DeferredActionInterface::IrqAck)
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
    cptr: sel4::cap::Unspecified,
    msg_info: sel4::MessageInfo,
}

impl PreparedDeferredAction {
    pub(crate) fn new(cptr: sel4::cap::Unspecified, msg_info: sel4::MessageInfo) -> Self {
        Self { cptr, msg_info }
    }

    pub(crate) fn cptr(&self) -> sel4::cap::Unspecified {
        self.cptr
    }

    pub(crate) fn msg_info(&self) -> sel4::MessageInfo {
        self.msg_info.clone() // TODO
    }
}

/// Utility type for implementing [`Handler::take_deferred_action`].
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

    pub fn defer_notify(&mut self, channel: Channel) -> Result<(), IrqAckError> {
        self.defer(DeferredAction::new_notify(channel))
    }

    pub fn defer_irq_ack(&mut self, channel: Channel) -> Result<(), IrqAckError> {
        self.defer(DeferredAction::new_irq_ack(channel))
    }
}

impl Default for DeferredActionSlot {
    fn default() -> Self {
        Self::new()
    }
}
