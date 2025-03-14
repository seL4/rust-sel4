//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::panic::PanicInfo;

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking::set_hook as set_outer_hook;
use sel4_panicking_env::debug_println;

pub use sel4_panicking::{catch_unwind, PanicHook};

use crate::pd_name;

static PANIC_HOOK: ImmediateSyncOnceCell<PanicHook> = ImmediateSyncOnceCell::new();

pub fn set_hook(hook: PanicHook) {
    PANIC_HOOK.set(hook).unwrap_or_else(|_| panic!())
}

fn get_hook() -> &'static PanicHook {
    const DEFAULT_HOOK: PanicHook = &default_hook;
    PANIC_HOOK.get().unwrap_or(&DEFAULT_HOOK)
}

fn default_hook(info: &PanicInfo) {
    debug_println!("{}: {}", pd_name().unwrap_or("?"), info);
}

fn outer_hook(info: &PanicInfo) {
    (get_hook())(info)
}

pub(crate) fn init_panicking() {
    set_outer_hook(&outer_hook)
}
