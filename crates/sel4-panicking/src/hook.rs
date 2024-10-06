//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking_env::debug_println;

use crate::ExternalPanicInfo;

/// Type for panic hooks.
///
/// See [`set_hook`].
pub type PanicHook = &'static (dyn Fn(&ExternalPanicInfo) + Send + Sync);

static PANIC_HOOK: ImmediateSyncOnceCell<PanicHook> = ImmediateSyncOnceCell::new();

/// Like `std::panic::set_hook`.
pub fn set_hook(hook: PanicHook) {
    PANIC_HOOK.set(hook).unwrap_or_else(|_| panic!())
}

pub(crate) fn get_hook() -> &'static PanicHook {
    const DEFAULT_HOOK: PanicHook = &default_hook;
    PANIC_HOOK.get().unwrap_or(&DEFAULT_HOOK)
}

fn default_hook(info: &ExternalPanicInfo) {
    debug_println!("{}", info);
}
