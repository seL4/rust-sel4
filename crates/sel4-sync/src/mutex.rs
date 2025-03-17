//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use core::marker::PhantomData;
use core::sync::atomic::{fence, AtomicIsize, Ordering};

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;

pub struct RawNotificationMutex<_T = ()> {
    inner: GenericRawMutex<sel4::cap::Notification>,
    phantom: PhantomData<_T>,
}

impl RawNotificationMutex {
    pub const fn new(nfn: sel4::cap::Notification) -> Self {
        Self {
            inner: GenericRawMutex::new(nfn),
            phantom: PhantomData,
        }
    }
}

unsafe impl<_T> lock_api::RawMutex for RawNotificationMutex<_T> {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    #[allow(unreachable_code)]
    #[allow(clippy::diverging_sub_expression)]
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = {
        let _: _T = unimplemented!();
        unimplemented!()
    };

    fn lock(&self) {
        self.inner.lock()
    }

    fn try_lock(&self) -> bool {
        self.inner.try_lock()
    }

    unsafe fn unlock(&self) {
        self.inner.unlock()
    }
}

pub struct DeferredRawNotificationMutex(GenericRawMutex<DeferredNotificationMutexSyncOps>);

impl DeferredRawNotificationMutex {
    pub const fn new() -> Self {
        Self(GenericRawMutex::new(DeferredNotificationMutexSyncOps::new()))
    }

    pub fn set_notification(
        &self,
        nfn: sel4::cap::Notification,
    ) -> Result<(), NotificationAlreadySetError> {
        self.0
            .sync_ops
            .inner
            .set(nfn)
            .map_err(|_| NotificationAlreadySetError(()))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NotificationAlreadySetError(());

impl Default for DeferredRawNotificationMutex {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl lock_api::RawMutex for DeferredRawNotificationMutex {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self::new();

    fn lock(&self) {
        self.0.lock()
    }

    fn try_lock(&self) -> bool {
        self.0.try_lock()
    }

    unsafe fn unlock(&self) {
        self.0.unlock()
    }
}

pub struct LazyRawNotificationMutex<F = fn() -> sel4::cap::Notification>(
    GenericRawMutex<LazyNotificationMutexSyncOps<F>>,
);

impl<F> LazyRawNotificationMutex<F> {
    pub const fn new(f: F) -> Self {
        Self(GenericRawMutex::new(LazyNotificationMutexSyncOps::new(f)))
    }
}

unsafe impl<F: Fn() -> sel4::cap::Notification> lock_api::RawMutex for LazyRawNotificationMutex<F> {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = unimplemented!();

    fn lock(&self) {
        self.0.lock()
    }

    fn try_lock(&self) -> bool {
        self.0.try_lock()
    }

    unsafe fn unlock(&self) {
        self.0.unlock()
    }
}

// // //

struct GenericRawMutex<O> {
    sync_ops: O,
    value: AtomicIsize,
}

impl<O> GenericRawMutex<O> {
    const fn new(sync_ops: O) -> Self {
        Self {
            sync_ops,
            value: AtomicIsize::new(1),
        }
    }
}

trait MutexSyncOps {
    fn signal(&self);
    fn wait(&self);
}

unsafe impl<O: MutexSyncOps> lock_api::RawMutex for GenericRawMutex<O> {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    #[allow(clippy::declare_interior_mutable_const)]
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

trait MutexSyncOpsWithNotification {
    fn notification(&self) -> sel4::cap::Notification;
}

impl<O: MutexSyncOpsWithNotification> MutexSyncOps for O {
    fn signal(&self) {
        self.notification().signal()
    }

    fn wait(&self) {
        let _badge = self.notification().wait();
    }
}

impl MutexSyncOpsWithNotification for sel4::cap::Notification {
    fn notification(&self) -> sel4::cap::Notification {
        *self
    }
}

struct DeferredNotificationMutexSyncOps {
    inner: ImmediateSyncOnceCell<sel4::cap::Notification>,
}

impl DeferredNotificationMutexSyncOps {
    const fn new() -> Self {
        Self {
            inner: ImmediateSyncOnceCell::new(),
        }
    }
}

impl MutexSyncOpsWithNotification for DeferredNotificationMutexSyncOps {
    fn notification(&self) -> sel4::cap::Notification {
        *self.inner.get().unwrap()
    }
}

struct LazyNotificationMutexSyncOps<F> {
    f: F,
    state: ImmediateSyncOnceCell<sel4::cap::Notification>,
}

impl<F> LazyNotificationMutexSyncOps<F> {
    const fn new(f: F) -> Self {
        Self {
            f,
            state: ImmediateSyncOnceCell::new(),
        }
    }
}

impl<F: Fn() -> sel4::cap::Notification> MutexSyncOpsWithNotification
    for LazyNotificationMutexSyncOps<F>
{
    fn notification(&self) -> sel4::cap::Notification {
        match self.state.get() {
            Some(nfn) => *nfn,
            None => {
                let _ = self.state.set((self.f)());
                *self.state.get().unwrap()
            }
        }
    }
}
