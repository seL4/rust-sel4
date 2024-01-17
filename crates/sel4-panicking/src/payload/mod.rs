//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::mem;

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod with_alloc;
        use with_alloc as whether_alloc;
    } else {
        mod without_alloc;
        use without_alloc as whether_alloc;
    }
}

pub use whether_alloc::*;

pub trait UpcastIntoPayload {
    fn upcast_into_payload(self) -> Payload;
}

pub const SMALL_PAYLOAD_MAX_SIZE: usize = 32;

#[allow(dead_code)]
#[inline(always)]
const fn check_small_payload_size<T: SmallPayload>() {
    struct Check<T>(T);

    impl<T> Check<T> {
        const CHECK: () = assert!(
            mem::size_of::<T>() <= SMALL_PAYLOAD_MAX_SIZE,
            "type is is too large to implement SmallPayload",
        );
    }

    Check::<T>::CHECK
}

pub trait SmallPayload {}

#[derive(Clone, Copy)]
pub(crate) struct NoPayload;

impl SmallPayload for NoPayload {}
