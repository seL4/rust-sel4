//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

#![no_std]

pub use lock_api;

mod mutex;

pub use mutex::{
    AbstractMutexSyncOps, DeferredNotificationMutexSyncOps, GenericRawMutex,
    IndirectNotificationMutexSyncOps, MutexSyncOps, MutexSyncOpsWithInteriorMutability,
    MutexSyncOpsWithNotification, PanickingMutexSyncOps, PanickingRawMutex,
};
