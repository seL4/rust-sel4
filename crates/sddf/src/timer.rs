//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_variables)]
#![allow(dead_code)]

use core::marker::PhantomData;
use core::num::Wrapping;
use core::ptr::NonNull;
use core::slice;

use sddf_sys as sys;
use sel4_shared_memory::access::*;
use sel4_shared_memory::{map_field, SharedMemoryPtr, SharedMemoryRef};

use crate::{common::*, Config};

type Result<T> = core::result::Result<T, PeerMisbehaviorError>;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ClientConfig(sys::timer_client_config);

unsafe impl Config for ClientConfig {
    fn is_magic_valid(&self) -> bool {
        self.0.magic == sys::SDDF_TIMER_MAGIC
    }
}

impl ClientConfig {
    pub fn driver_id(&self) -> u8 {
        self.0.driver_id
    }
}
