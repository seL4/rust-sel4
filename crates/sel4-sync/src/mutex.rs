//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use core::sync::atomic::{fence, AtomicBool, AtomicIsize, Ordering};

use sel4::Notification;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;

pub struct PanickingRawMutex {
    locked: AtomicBool,
}

unsafe impl lock_api::RawMutex for PanickingRawMutex {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self {
        locked: AtomicBool::new(false),
    };

    fn lock(&self) {
        if !self.try_lock() {
            panic!("lock contention")
        }
    }

    fn try_lock(&self) -> bool {
        let was_locked = self.locked.swap(true, Ordering::Acquire);
        !was_locked
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release)
    }
}

pub struct GenericRawMutex<O> {
    sync_ops: O,
    value: AtomicIsize,
}

impl<O> GenericRawMutex<O> {
    pub const fn new(sync_ops: O) -> Self {
        Self {
            sync_ops,
            value: AtomicIsize::new(1),
        }
    }
}

impl<O: MutexSyncOpsWithInteriorMutability> GenericRawMutex<O> {
    pub fn modify(&self, input: O::ModifyInput) -> O::ModifyOutput {
        self.sync_ops.modify(input)
    }
}

unsafe impl<O: MutexSyncOps> lock_api::RawMutex for GenericRawMutex<O> {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    const INIT: Self = unimplemented!();

    fn lock(&self) {
        let old_value = self.value.fetch_sub(1, Ordering::Acquire);
        if old_value <= 0 {
            self.sync_ops.wait();
            fence(Ordering::Acquire);
        }
    }

    fn try_lock(&self) -> bool {
        unimplemented!()
    }

    unsafe fn unlock(&self) {
        let old_value = self.value.fetch_add(1, Ordering::Release);
        if old_value < 0 {
            self.sync_ops.signal();
        }
    }
}

pub trait MutexSyncOps {
    fn signal(&self);
    fn wait(&self);
}

pub trait MutexSyncOpsWithInteriorMutability {
    type ModifyInput;
    type ModifyOutput;

    fn modify(&self, input: Self::ModifyInput) -> Self::ModifyOutput;
}

pub trait MutexSyncOpsWithNotification {
    fn notification(&self) -> Notification;
}

impl<O: MutexSyncOpsWithNotification> MutexSyncOps for O {
    fn signal(&self) {
        self.notification().signal()
    }

    fn wait(&self) {
        let _badge = self.notification().wait();
    }
}

impl MutexSyncOpsWithNotification for Notification {
    fn notification(&self) -> Notification {
        *self
    }
}

pub struct DeferredNotificationMutexSyncOps {
    inner: ImmediateSyncOnceCell<Notification>,
}

impl DeferredNotificationMutexSyncOps {
    pub const fn new() -> Self {
        Self {
            inner: ImmediateSyncOnceCell::new(),
        }
    }
}

impl MutexSyncOpsWithNotification for DeferredNotificationMutexSyncOps {
    fn notification(&self) -> Notification {
        *self.inner.get().unwrap()
    }
}

impl MutexSyncOpsWithInteriorMutability for DeferredNotificationMutexSyncOps {
    type ModifyInput = Notification;
    type ModifyOutput = ();

    fn modify(&self, input: Self::ModifyInput) -> Self::ModifyOutput {
        self.inner.set(input).unwrap()
    }
}

pub struct IndirectNotificationMutexSyncOps<T> {
    get_notification: T,
}

impl<T: Fn() -> Notification> IndirectNotificationMutexSyncOps<T> {
    pub const fn new(get_notification: T) -> Self {
        Self { get_notification }
    }
}

impl<T: Fn() -> Notification> MutexSyncOpsWithNotification for IndirectNotificationMutexSyncOps<T> {
    fn notification(&self) -> Notification {
        (self.get_notification)()
    }
}

pub struct AbstractMutexSyncOps<T, U> {
    pub signal: T,
    pub wait: U,
}

impl<T: Fn(), U: Fn()> AbstractMutexSyncOps<T, U> {
    pub const fn new(signal: T, wait: U) -> Self {
        Self { signal, wait }
    }
}

impl<T: Fn(), U: Fn()> MutexSyncOps for AbstractMutexSyncOps<T, U> {
    fn signal(&self) {
        (self.signal)()
    }

    fn wait(&self) {
        (self.wait)()
    }
}

pub struct PanickingMutexSyncOps(());

impl PanickingMutexSyncOps {
    pub const fn new() -> Self {
        Self(())
    }
}

impl MutexSyncOps for PanickingMutexSyncOps {
    fn signal(&self) {
        panic!("unexpected contention: signal")
    }

    fn wait(&self) {
        panic!("unexpected contention: wait")
    }
}
