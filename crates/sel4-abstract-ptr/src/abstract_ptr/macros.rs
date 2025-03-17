//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2020 Philipp Oppermann
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

#[macro_export]
macro_rules! map_field {
    ($volatile:ident.$($place:ident).+) => {{
        // Simulate creating a reference to the field. This is done to make
        // sure that the field is not potentially unaligned. The body of the
        // if statement will never be executed, so it can never cause any UB.
        if false {
            let _ref_to_field = &(unsafe { &*$volatile.as_raw_ptr().as_ptr() }).$($place).+;
        }

        unsafe {
            $volatile.map(|ptr| {
                core::ptr::NonNull::new(core::ptr::addr_of_mut!((*ptr.as_ptr()).$($place).+)).unwrap()
            })
        }
    }};
}
