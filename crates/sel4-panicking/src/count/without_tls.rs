use core::sync::atomic::{AtomicBool, Ordering};

use sel4_panicking_env::abort;

static PANICKING: AtomicBool = AtomicBool::new(false);

pub(crate) fn count_panic() {
    if PANICKING.load(Ordering::SeqCst) {
        abort!("recursive panic encountered");
    }
}

pub(crate) fn count_panic_caught() {
    PANICKING.store(false, Ordering::SeqCst);
}
