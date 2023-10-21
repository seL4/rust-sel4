//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::panic::PanicInfo;

use crate::arch::{Arch, ArchImpl};

#[panic_handler]
extern "C" fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    ArchImpl::idle()
}
