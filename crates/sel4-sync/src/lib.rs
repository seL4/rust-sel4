#![no_std]
#![feature(const_trait_impl)]
#![feature(derive_const)]

mod mutex;

pub use mutex::{
    AbstractMutexSyncOps, DeferredMutex, DeferredMutexGuard, DeferredNotificationMutexSyncOps,
    GenericMutex, GenericMutexGuard, IndirectNotificationMutexSyncOps, Mutex, MutexGuard,
    MutexSyncOps, MutexSyncOpsWithInteriorMutability, MutexSyncOpsWithNotification,
    PanickingMutexSyncOps,
};
