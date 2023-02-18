use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{fence, AtomicIsize, Ordering};

use sel4::Notification;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;

pub trait MutexSyncOps {
    fn signal(&self);
    fn wait(&self);
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

pub trait MutexSyncOpsWithInteriorMutability {
    type ModifyInput;
    type ModifyOutput;

    fn modify(&self, input: Self::ModifyInput) -> Self::ModifyOutput;
}

struct RawGenericMutex<O> {
    sync_ops: O,
    value: AtomicIsize,
}

impl<O> RawGenericMutex<O> {
    pub const fn new(sync_ops: O) -> Self {
        Self {
            sync_ops,
            value: AtomicIsize::new(1),
        }
    }
}

impl<O: MutexSyncOps> RawGenericMutex<O> {
    fn lock(&self) {
        let old_value = self.value.fetch_sub(1, Ordering::Acquire);
        if old_value <= 0 {
            self.sync_ops.wait();
            fence(Ordering::Acquire);
        }
    }

    fn unlock(&self) {
        let old_value = self.value.fetch_add(1, Ordering::Release);
        if old_value < 0 {
            self.sync_ops.signal();
        }
    }
}

pub struct GenericMutex<O, T: ?Sized> {
    raw: RawGenericMutex<O>,
    data: UnsafeCell<T>,
}

unsafe impl<O, T: ?Sized + Send> Send for GenericMutex<O, T> {}
unsafe impl<O, T: ?Sized + Send> Sync for GenericMutex<O, T> {}

pub struct GenericMutexGuard<'a, O: MutexSyncOps, T: ?Sized + 'a> {
    mutex: &'a GenericMutex<O, T>,
}

impl<O, T> GenericMutex<O, T> {
    pub const fn new(sync_ops: O, val: T) -> Self {
        Self {
            raw: RawGenericMutex::new(sync_ops),
            data: UnsafeCell::new(val),
        }
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<O: MutexSyncOps, T> GenericMutex<O, T> {
    unsafe fn guard(&self) -> GenericMutexGuard<'_, O, T> {
        GenericMutexGuard { mutex: self }
    }

    pub fn lock(&self) -> GenericMutexGuard<'_, O, T> {
        self.raw.lock();
        unsafe { self.guard() }
    }
}

impl<O: MutexSyncOpsWithInteriorMutability, T> GenericMutex<O, T> {
    pub fn modify_sync_ops(&self, input: O::ModifyInput) -> O::ModifyOutput {
        self.raw.sync_ops.modify(input)
    }
}

impl<'a, O: MutexSyncOps, T: ?Sized + 'a> GenericMutexGuard<'a, O, T> {
    pub fn mutex(this: &Self) -> &'a GenericMutex<O, T> {
        this.mutex
    }
}

impl<'a, O: MutexSyncOps, T: ?Sized + 'a> Deref for GenericMutexGuard<'a, O, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, O: MutexSyncOps, T: ?Sized + 'a> DerefMut for GenericMutexGuard<'a, O, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, O: MutexSyncOps, T: ?Sized + 'a> Drop for GenericMutexGuard<'a, O, T> {
    fn drop(&mut self) {
        self.mutex.raw.unlock();
    }
}

pub type Mutex<T> = GenericMutex<Notification, T>;
pub type MutexGuard<'a, T> = GenericMutexGuard<'a, Notification, T>;

impl MutexSyncOpsWithNotification for Notification {
    fn notification(&self) -> Notification {
        *self
    }
}

pub type DeferredMutex<T> = GenericMutex<DeferredNotificationMutexSyncOps, T>;
pub type DeferredMutexGuard<'a, T> = GenericMutexGuard<'a, DeferredNotificationMutexSyncOps, T>;

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

#[derive_const(Default)]
pub struct PanickingMutexSyncOps(());

impl PanickingMutexSyncOps {
    pub const fn new() -> Self {
        Self::default()
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
