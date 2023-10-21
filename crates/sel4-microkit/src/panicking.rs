//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking::set_hook as set_outer_hook;
use sel4_panicking_env::debug_println;

pub use sel4_panicking::{
    catch_unwind, panic_any, ExternalPanicInfo, FitsWithinSmallPayload, PanicHook, Payload,
    SmallPayloadValue, UpcastIntoPayload,
};

use crate::pd_name;

static PANIC_HOOK: ImmediateSyncOnceCell<PanicHook> = ImmediateSyncOnceCell::new();

pub fn set_hook(hook: PanicHook) {
    PANIC_HOOK.set(hook).unwrap_or_else(|_| panic!())
}

fn get_hook() -> &'static PanicHook {
    const DEFAULT_HOOK: PanicHook = &default_hook;
    PANIC_HOOK.get().unwrap_or(&DEFAULT_HOOK)
}

fn default_hook(info: &ExternalPanicInfo) {
    debug_println!("{}: {}", pd_name(), info);
}

fn outer_hook(info: &ExternalPanicInfo) {
    (get_hook())(info)
}

pub(crate) fn init_panicking() {
    set_outer_hook(&outer_hook)
}

// // //

#[no_mangle]
#[allow(unused_variables)]
fn sel4_runtime_debug_put_char(c: u8) {
    #[sel4::sel4_cfg(PRINTING)]
    {
        sel4::debug_put_char(c as core::ffi::c_char)
    }
}
