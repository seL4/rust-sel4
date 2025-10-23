//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

#![no_std]

use core::mem;

#[cfg(feature = "alloc")]
extern crate alloc;

pub type StaticThreadEntryFn = extern "C" fn(arg0: sel4::Word, arg1: sel4::Word);

#[derive(Copy, Clone, Debug)]
pub struct StaticThread(sel4::cap::Endpoint);

impl StaticThread {
    pub fn new(endpoint: sel4::cap::Endpoint) -> Self {
        Self(endpoint)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn recv_and_run(
        endpoint: sel4::cap::Endpoint,
        reply_authority: sel4::ReplyAuthority,
    ) {
        let sel4::RecvWithMRs {
            msg: [entry_vaddr, entry_arg0, entry_arg1, ..],
            ..
        } = endpoint.recv_with_mrs(reply_authority);
        let entry_fn: StaticThreadEntryFn = unsafe { mem::transmute(entry_vaddr) };
        (entry_fn)(entry_arg0, entry_arg1);
    }
}

impl From<sel4::cap::Endpoint> for StaticThread {
    fn from(endpoint: sel4::cap::Endpoint) -> Self {
        Self::new(endpoint)
    }
}

#[cfg(feature = "alloc")]
mod when_alloc {
    use alloc::boxed::Box;
    use core::panic::UnwindSafe;

    use sel4_panicking::catch_unwind;

    use crate::StaticThread;

    impl StaticThread {
        pub fn start(&self, f: impl FnOnce() + Send + 'static) {
            let b: Box<Box<dyn FnOnce() + 'static>> = Box::new(Box::new(f));
            let f_arg = Box::into_raw(b);
            self.0.send_with_mrs(
                sel4::MessageInfoBuilder::default().length(3).build(),
                [entry as usize as sel4::Word, f_arg as sel4::Word, 0],
            );
        }
    }

    extern "C" fn entry(f_arg: sel4::Word) {
        let f = unsafe { Box::from_raw(f_arg as *mut Box<dyn FnOnce() + UnwindSafe>) };
        let _ = catch_unwind(f);
    }
}
