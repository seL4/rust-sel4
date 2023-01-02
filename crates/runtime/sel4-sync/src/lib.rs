#![no_std]
#![feature(never_type)]

mod mutex;

pub use mutex::{
    AbstractMutexSyncOps, DeferredMutex, DeferredMutexGuard, DeferredNotificationMutexSyncOps,
    GenericMutex, GenericMutexGuard, IndirectNotificationMutexSyncOps, Mutex, MutexGuard,
    MutexSyncOps, MutexSyncOpsWithInteriorMutability, MutexSyncOpsWithNotification,
};
