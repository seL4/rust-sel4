//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use sddf_sys::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// TODO name enums upstream
pub use _bindgen_ty_1 as fs_open_flags;
pub use _bindgen_ty_2 as fs_status;
pub use _bindgen_ty_3 as fs_cmd_;

// Check _bindgen_ty_* order
const _: () = {
    let _ = (
        fs_open_flags::FS_OPEN_FLAGS_CREATE,
        fs_status::FS_STATUS_ALLOCATION_ERROR,
        fs_cmd_::FS_CMD_DEINITIALISE,
    );
};
