use core::panic::PanicInfo;

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking_env::debug_println;

pub type PanicHook = &'static (dyn Fn(Option<&PanicInfo>) + Send + Sync);

static PANIC_HOOK: ImmediateSyncOnceCell<PanicHook> = ImmediateSyncOnceCell::new();

pub fn set_hook(hook: PanicHook) {
    PANIC_HOOK.set(hook).unwrap_or_else(|_| panic!())
}

pub(crate) fn get_hook() -> &'static PanicHook {
    const DEFAULT_HOOK: PanicHook = &default_hook;
    PANIC_HOOK.get().unwrap_or(&DEFAULT_HOOK)
}

fn default_hook(info: Option<&PanicInfo>) {
    if let Some(info) = info {
        debug_println!("{}", info);
    }
}
