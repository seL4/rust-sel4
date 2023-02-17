use core::cell::Cell;

use sel4_panicking_env::abort;

// TODO consider supporting nested panics
#[thread_local]
static PANIC_COUNT: Cell<usize> = Cell::new(0);

const MAX_PANIC_DEPTH: usize = if cfg!(feature = "alloc") { 2 } else { 1 };

pub(crate) fn count_panic() {
    if PANIC_COUNT.get() >= MAX_PANIC_DEPTH {
        abort!("maximum panic depth of {MAX_PANIC_DEPTH} reached");
    }
    PANIC_COUNT.update(|count| count + 1);
}

pub(crate) fn count_panic_caught() {
    PANIC_COUNT.update(|count| count - 1);
}
