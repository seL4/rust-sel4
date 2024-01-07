//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::Cell;

use sel4_panicking_env::abort;

#[thread_local]
static PANIC_COUNT: Cell<usize> = Cell::new(0);

const MAX_PANIC_DEPTH: usize = if cfg!(feature = "alloc") { 3 } else { 1 };

pub(crate) fn count_panic() {
    if PANIC_COUNT.get() == MAX_PANIC_DEPTH {
        abort!("maximum panic depth of {MAX_PANIC_DEPTH} exceeded");
    }
    update(&PANIC_COUNT, |count| count + 1);
}

pub(crate) fn count_panic_caught() {
    update(&PANIC_COUNT, |count| count - 1);
}

// NOTE(rustc_wishlist) until #![feature(cell_update)] stabilizes
fn update<T: Copy>(cell: &Cell<T>, f: impl FnOnce(T) -> T) -> T {
    let old = cell.get();
    let new = f(old);
    cell.set(new);
    new
}
