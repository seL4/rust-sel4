use core::cell::Cell;

use sel4_panicking_env::abort;

// TODO consider supporting nested panics
#[thread_local]
static PANIC_COUNT: Cell<usize> = Cell::new(0);

pub(crate) fn count_panic() {
    if PANIC_COUNT.get() >= 1 {
        abort!("thread panicked while processing panic. aborting.");
    }
    PANIC_COUNT.set(1);
}

pub(crate) fn count_panic_caught() {
    PANIC_COUNT.set(0);
}
