//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2020 Philipp Oppermann
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

/// Provides safe field projection for abstract pointers referencing structs.
///
/// ## Examples
///
/// Accessing a struct field:
///
/// ```ignore
/// use sel4_abstract_ptr::{AbstractPtr, map_field};
///
/// struct Example { field_1: u32, field_2: u8, }
/// let mut value = Example { field_1: 15, field_2: 255 };
/// let ptr = unsafe { AbstractPtr::new((&mut value).into()) };
///
/// // construct an abstract reference to a field
/// let field_2 = map_field!(ptr.field_2);
/// assert_eq!(field_2.read(), 255);
/// ```
///
/// Creating `AbstractPtr`s to unaligned field in packed structs is not allowed:
/// ```ignore
/// use sel4_abstract_ptr::{AbstractPtr, map_field};
///
/// #[repr(packed)]
/// struct Example { field_1: u8, field_2: usize, }
/// let mut value = Example { field_1: 15, field_2: 255 };
/// let ptr = unsafe { AbstractPtr::new((&mut value).into()) };
///
/// // Constructing an abstract reference to an unaligned field doesn't compile.
/// let field_2 = map_field!(ptr.field_2);
/// ```
#[macro_export]
macro_rules! map_field {
    ($abstract_ptr:ident.$($place:ident).+) => {{
        // Simulate creating a reference to the field. This is done to make
        // sure that the field is not potentially unaligned. The body of the
        // if statement will never be executed, so it can never cause any UB.
        if false {
            let _ref_to_field = &(unsafe { &*$abstract_ptr.as_raw_ptr().as_ptr() }).$($place).+;
        }

        unsafe {
            $abstract_ptr.map(|ptr| {
                core::ptr::NonNull::new(core::ptr::addr_of_mut!((*ptr.as_ptr()).$($place).+)).unwrap()
            })
        }
    }};
}
