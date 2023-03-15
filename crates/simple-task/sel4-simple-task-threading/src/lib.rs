#![no_std]

use core::mem;

use sel4::{Endpoint, RecvWithMRs};

#[cfg(feature = "alloc")]
extern crate alloc;

pub type StaticThreadEntryFn = extern "C" fn(arg0: u64, arg1: u64);

#[derive(Copy, Clone, Debug)]
pub struct StaticThread(Endpoint);

impl StaticThread {
    pub fn new(endpoint: Endpoint) -> Self {
        Self(endpoint)
    }

    pub unsafe fn recv_and_run(endpoint: Endpoint) {
        let RecvWithMRs {
            msg: [entry_vaddr, entry_arg0, entry_arg1, ..],
            ..
        } = endpoint.recv_with_mrs(());
        let entry_fn: StaticThreadEntryFn = mem::transmute(entry_vaddr);
        (entry_fn)(entry_arg0, entry_arg1);
    }
}

impl From<Endpoint> for StaticThread {
    fn from(endpoint: Endpoint) -> Self {
        Self::new(endpoint)
    }
}

#[cfg(feature = "alloc")]
mod when_alloc {
    use alloc::boxed::Box;

    use sel4::Word;
    use sel4_panicking::catch_unwind;

    use crate::StaticThread;

    impl StaticThread {
        pub fn start(&self, f: impl FnOnce() + Send + 'static) {
            let b: Box<Box<dyn FnOnce() + 'static>> = Box::new(Box::new(f));
            let f_arg = Box::into_raw(b);
            self.0.send_with_mrs(
                sel4::MessageInfoBuilder::default().length(3).build(),
                [entry as Word, f_arg as Word, 0],
            );
        }
    }

    extern "C" fn entry(f_arg: u64) {
        let f = unsafe { Box::from_raw(f_arg as *mut Box<dyn FnOnce()>) };
        let _ = catch_unwind(|| f());
    }
}
