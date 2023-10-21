//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub use sel4::{local_cptr::*, CPtr};
pub use sel4_simple_task_threading::StaticThread;

use sel4::{Badge, CPtrBits, CapType, LocalCPtr};

use crate::{ConfigBadge, ConfigCPtr, ConfigCPtrBits};

pub trait WrappedCPtr {
    fn wrap(bits: CPtrBits) -> Self;
}

impl WrappedCPtr for CPtr {
    fn wrap(bits: CPtrBits) -> Self {
        Self::from_bits(bits)
    }
}

impl<T: CapType> WrappedCPtr for LocalCPtr<T> {
    fn wrap(bits: CPtrBits) -> Self {
        Self::from_bits(bits)
    }
}

impl WrappedCPtr for StaticThread {
    fn wrap(bits: CPtrBits) -> Self {
        Self::new(Endpoint::from_bits(bits))
    }
}

impl ConfigBadge {
    pub fn get(&self) -> Badge {
        self.0
    }
}

impl From<Badge> for ConfigBadge {
    fn from(badge: Badge) -> Self {
        Self::new(badge)
    }
}

impl ConfigCPtrBits {
    pub fn get(&self) -> CPtrBits {
        self.0
    }
}

impl<T: WrappedCPtr> ConfigCPtr<T> {
    pub fn get(&self) -> T {
        T::wrap(self.bits.get())
    }
}
