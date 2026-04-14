//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::marker::PhantomData;

use sel4_simple_task_threading::StaticThread;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigCPtr<T> {
    phantom: PhantomData<T>,
    bits: sel4::CPtrBits,
}

impl<T> ConfigCPtr<T> {
    pub fn new(bits: sel4::CPtrBits) -> Self {
        Self {
            phantom: PhantomData,
            bits,
        }
    }
}

impl<T: WrappedCPtr> ConfigCPtr<T> {
    pub fn get(&self) -> T {
        T::wrap(self.bits)
    }
}

pub trait WrappedCPtr {
    fn wrap(bits: sel4::CPtrBits) -> Self;
}

impl WrappedCPtr for sel4::CPtr {
    fn wrap(bits: sel4::CPtrBits) -> Self {
        Self::from_bits(bits)
    }
}

impl<T: sel4::CapType> WrappedCPtr for sel4::Cap<T> {
    fn wrap(bits: sel4::CPtrBits) -> Self {
        Self::from_bits(bits)
    }
}

impl WrappedCPtr for StaticThread {
    fn wrap(bits: sel4::CPtrBits) -> Self {
        Self::new(sel4::cap::Endpoint::from_bits(bits))
    }
}
