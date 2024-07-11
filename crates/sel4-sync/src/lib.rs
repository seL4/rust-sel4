//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

#![no_std]

pub use lock_api;

pub use sel4_sync_trivial::*;

mod mutex;

pub use mutex::{
    DeferredNotificationMutexSyncOps, GenericRawMutex, MutexSyncOps,
    MutexSyncOpsWithInteriorMutability, MutexSyncOpsWithNotification,
};
