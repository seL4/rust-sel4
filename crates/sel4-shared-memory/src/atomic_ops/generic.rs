//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Rust project contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::intrinsics;

use paste::paste;

use super::ordering::OrderingExhaustive as Ordering;

use Ordering::*;

macro_rules! with_ordering {
    ($ord:ident, $prefix:ident $args:tt) => {
        paste! {
            intrinsics::[<$prefix _ $ord:lower>]$args
        }
    };
}

macro_rules! with_orderings {
    ($set_ord:ident, $fetch_ord:ident, $prefix:ident $args:tt) => {
        paste! {
            intrinsics::[<$prefix _ $set_ord:lower _ $fetch_ord:lower>]$args
        }
    };
}

macro_rules! match_ordering {
    {
        $prefix:ident $args:tt, match $ord_expr:expr,
            [
                $($good_ord:ident,)*
            ]
            {
                $($bad_ord:ident => $bad_ord_body:expr,)*
            }
    } => {
        match $ord_expr {
            $($good_ord => with_ordering!($good_ord, $prefix $args),)*
            $($bad_ord => $bad_ord_body,)*
        }
    };
}

macro_rules! match_ordering_all {
    {
        $prefix:ident $args:tt, match $ord_expr:expr,
    } => {
        match_ordering! {
            $prefix $args, match $ord_expr,
                [
                    Relaxed,
                    Acquire,
                    Release,
                    AcqRel,
                    SeqCst,
                ]
                {
                }
        }
    };
}

macro_rules! match_orderings {
    {
        $prefix:ident $args:tt, match $ords_expr:expr,
            [
                $(($good_set_ord:ident, $good_fetch_ord:ident),)*
            ]
            {
                $($bad_ords:pat => $bad_ords_body:expr,)*
            }
    } => {
        match $ords_expr {
            $(($good_set_ord, $good_fetch_ord) => with_orderings!($good_set_ord, $good_fetch_ord, $prefix $args),)*
            $($bad_ords => $bad_ords_body,)*
        }
    };
}

#[inline]
pub(crate) unsafe fn atomic_store<T: Copy>(dst: *mut T, val: T, order: Ordering) {
    // SAFETY: the caller must uphold the safety contract for `atomic_store`.
    unsafe {
        match_ordering! {
            atomic_store(dst, val), match order,
                [
                    Relaxed,
                    Release,
                    SeqCst,
                ]
                {
                    Acquire => panic!("there is no such thing as an acquire store"),
                    AcqRel => panic!("there is no such thing as an acquire-release store"),
                }
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_load<T: Copy>(dst: *const T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_load`.
    unsafe {
        match_ordering! {
            atomic_load(dst), match order,
                [
                    Relaxed,
                    Acquire,
                    SeqCst,
                ]
                {
                    Release => panic!("there is no such thing as a release load"),
                    AcqRel => panic!("there is no such thing as an acquire-release load"),
                }
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_swap<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_swap`.
    unsafe {
        match_ordering_all! {
            atomic_xchg(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_add<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_add`.
    unsafe {
        match_ordering_all! {
            atomic_xadd(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_sub<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_sub`.
    unsafe {
        match_ordering_all! {
            atomic_xsub(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_compare_exchange<T: Copy>(
    dst: *mut T,
    old: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
    // SAFETY: the caller must uphold the safety contract for `atomic_compare_exchange`.
    let (val, ok) = unsafe {
        match_orderings! {
            atomic_cxchg(dst, old, new), match (success, failure),
                [
                    (Relaxed, Relaxed),
                    (Relaxed, Acquire),
                    (Relaxed, SeqCst),
                    (Acquire, Relaxed),
                    (Acquire, Acquire),
                    (Acquire, SeqCst),
                    (Release, Relaxed),
                    (Release, Acquire),
                    (Release, SeqCst),
                    (AcqRel, Relaxed),
                    (AcqRel, Acquire),
                    (AcqRel, SeqCst),
                    (SeqCst, Relaxed),
                    (SeqCst, Acquire),
                    (SeqCst, SeqCst),
                ]
                {
                    (_, AcqRel) => panic!("there is no such thing as an acquire-release failure ordering"),
                    (_, Release) => panic!("there is no such thing as a release failure ordering"),
                }
        }
    };
    if ok {
        Ok(val)
    } else {
        Err(val)
    }
}

#[inline]
pub(crate) unsafe fn atomic_compare_exchange_weak<T: Copy>(
    dst: *mut T,
    old: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
    // SAFETY: the caller must uphold the safety contract for `atomic_compare_exchange_weak`.
    let (val, ok) = unsafe {
        match_orderings! {
            atomic_cxchgweak(dst, old, new), match (success, failure),
                [
                    (Relaxed, Relaxed),
                    (Relaxed, Acquire),
                    (Relaxed, SeqCst),
                    (Acquire, Relaxed),
                    (Acquire, Acquire),
                    (Acquire, SeqCst),
                    (Release, Relaxed),
                    (Release, Acquire),
                    (Release, SeqCst),
                    (AcqRel, Relaxed),
                    (AcqRel, Acquire),
                    (AcqRel, SeqCst),
                    (SeqCst, Relaxed),
                    (SeqCst, Acquire),
                    (SeqCst, SeqCst),
                ]
                {
                    (_, AcqRel) => panic!("there is no such thing as an acquire-release failure ordering"),
                    (_, Release) => panic!("there is no such thing as a release failure ordering"),
                }
        }
    };
    if ok {
        Ok(val)
    } else {
        Err(val)
    }
}

#[inline]
pub(crate) unsafe fn atomic_and<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_and`
    unsafe {
        match_ordering_all! {
            atomic_and(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_nand<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_nand`
    unsafe {
        match_ordering_all! {
            atomic_nand(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_or<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_or`
    unsafe {
        match_ordering_all! {
            atomic_or(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_xor<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_xor`
    unsafe {
        match_ordering_all! {
            atomic_xor(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_max<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_max`
    unsafe {
        match_ordering_all! {
            atomic_max(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_min<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_min`
    unsafe {
        match_ordering_all! {
            atomic_min(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_umax<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_umax`
    unsafe {
        match_ordering_all! {
            atomic_umax(dst, val), match order,
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_umin<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_umin`
    unsafe {
        match_ordering_all! {
            atomic_umin(dst, val), match order,
        }
    }
}
