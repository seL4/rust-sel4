//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::mem;

use sel4::{Endpoint, RecvWithMRs, ReplyAuthority, Word};

#[cfg(feature = "alloc")]
extern crate alloc;

pub type StaticThreadEntryFn = extern "C" fn(arg0: Word, arg1: Word);

#[derive(Copy, Clone, Debug)]
pub struct StaticThread(Endpoint);

impl StaticThread {
    pub fn new(endpoint: Endpoint) -> Self {
        Self(endpoint)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn recv_and_run(endpoint: Endpoint, reply_authority: ReplyAuthority) {
        let RecvWithMRs {
            msg: [entry_vaddr, entry_arg0, entry_arg1, ..],
            ..
        } = endpoint.recv_with_mrs(reply_authority);
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
                [entry as usize as Word, f_arg as Word, 0],
            );
        }
    }

    extern "C" fn entry(f_arg: Word) {
        let f = unsafe { Box::from_raw(f_arg as *mut Box<dyn FnOnce()>) };
        let _ = catch_unwind(f);
    }
}
