//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use core::marker::PhantomData;
use core::sync::atomic::{AtomicIsize, Ordering, fence};

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

pub struct RawDeferredNotificationMutex(GenericRawMutex<DeferredNotification>);

impl RawDeferredNotificationMutex {
    pub const fn new() -> Self {
        Self(GenericRawMutex::new(DeferredNotification::new()))
    }

    pub fn set_notification(
        &self,
        nfn: sel4::cap::Notification,
    ) -> Result<(), NotificationAlreadySetError> {
        self.0.nfn.set_notification(nfn)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NotificationAlreadySetError(());

impl Default for RawDeferredNotificationMutex {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl lock_api::RawMutex for RawDeferredNotificationMutex {
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

pub struct RawLazyNotificationMutex<F = fn() -> sel4::cap::Notification>(
    GenericRawMutex<LazyNotification<F>>,
);

impl<F> RawLazyNotificationMutex<F> {
    pub const fn new(f: F) -> Self {
        Self(GenericRawMutex::new(LazyNotification::new(f)))
    }
}

unsafe impl<F: Fn() -> sel4::cap::Notification> lock_api::RawMutex for RawLazyNotificationMutex<F> {
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

struct GenericRawMutex<T> {
    nfn: T,
    value: AtomicIsize,
}

impl<T> GenericRawMutex<T> {
    const fn new(nfn: T) -> Self {
        Self {
            nfn,
            value: AtomicIsize::new(1),
        }
    }
}

trait GetNotification {
    fn get_notification(&self) -> sel4::cap::Notification;
}

impl<T: GetNotification> GenericRawMutex<T> {
    fn lock(&self) {
        let old_value = self.value.fetch_sub(1, Ordering::Acquire);
        if old_value <= 0 {
            let _badge = self.nfn.get_notification().wait();
            fence(Ordering::Acquire);
        }
    }

    fn try_lock(&self) -> bool {
        unimplemented!()
    }

    unsafe fn unlock(&self) {
        let old_value = self.value.fetch_add(1, Ordering::Release);
        if old_value < 0 {
            self.nfn.get_notification().signal();
        }
    }
}

impl GetNotification for sel4::cap::Notification {
    fn get_notification(&self) -> sel4::cap::Notification {
        *self
    }
}

struct DeferredNotification {
    inner: ImmediateSyncOnceCell<sel4::cap::Notification>,
}

impl DeferredNotification {
    const fn new() -> Self {
        Self {
            inner: ImmediateSyncOnceCell::new(),
        }
    }

    fn set_notification(
        &self,
        nfn: sel4::cap::Notification,
    ) -> Result<(), NotificationAlreadySetError> {
        self.inner
            .set(nfn)
            .map_err(|_| NotificationAlreadySetError(()))
    }
}

impl GetNotification for DeferredNotification {
    fn get_notification(&self) -> sel4::cap::Notification {
        *self.inner.get().unwrap()
    }
}

struct LazyNotification<F> {
    f: F,
    state: ImmediateSyncOnceCell<sel4::cap::Notification>,
}

impl<F> LazyNotification<F> {
    const fn new(f: F) -> Self {
        Self {
            f,
            state: ImmediateSyncOnceCell::new(),
        }
    }
}

impl<F: Fn() -> sel4::cap::Notification> GetNotification for LazyNotification<F> {
    fn get_notification(&self) -> sel4::cap::Notification {
        match self.state.get() {
            Some(nfn) => *nfn,
            None => {
                let _ = self.state.set((self.f)());
                *self.state.get().unwrap()
            }
        }
    }
}
