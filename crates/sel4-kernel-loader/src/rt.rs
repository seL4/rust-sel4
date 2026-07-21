//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::panic::PanicInfo;

use crate::arch::{Arch, ArchImpl};
use crate::fmt::debug_println;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    debug_println!("{info}");
    ArchImpl::idle()
}
