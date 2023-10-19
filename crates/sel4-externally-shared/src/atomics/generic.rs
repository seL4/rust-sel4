use core::intrinsics;

use super::OrderingExhaustive as Ordering;

use Ordering::*;

#[inline]
pub(crate) unsafe fn atomic_store<T: Copy>(dst: *mut T, val: T, order: Ordering) {
    // SAFETY: the caller must uphold the safety contract for `atomic_store`.
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_store_relaxed(dst, val),
            Release => intrinsics::atomic_store_release(dst, val),
            SeqCst => intrinsics::atomic_store_seqcst(dst, val),
            Acquire => panic!("there is no such thing as an acquire store"),
            AcqRel => panic!("there is no such thing as an acquire-release store"),
        }
    }
}

#[inline]
pub(crate) unsafe fn atomic_load<T: Copy>(dst: *const T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_load`.
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_load_relaxed(dst),
            Acquire => intrinsics::atomic_load_acquire(dst),
            SeqCst => intrinsics::atomic_load_seqcst(dst),
            Release => panic!("there is no such thing as a release load"),
            AcqRel => panic!("there is no such thing as an acquire-release load"),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_swap<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_swap`.
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_xchg_relaxed(dst, val),
            Acquire => intrinsics::atomic_xchg_acquire(dst, val),
            Release => intrinsics::atomic_xchg_release(dst, val),
            AcqRel => intrinsics::atomic_xchg_acqrel(dst, val),
            SeqCst => intrinsics::atomic_xchg_seqcst(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_add<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_add`.
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_xadd_relaxed(dst, val),
            Acquire => intrinsics::atomic_xadd_acquire(dst, val),
            Release => intrinsics::atomic_xadd_release(dst, val),
            AcqRel => intrinsics::atomic_xadd_acqrel(dst, val),
            SeqCst => intrinsics::atomic_xadd_seqcst(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_sub<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_sub`.
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_xsub_relaxed(dst, val),
            Acquire => intrinsics::atomic_xsub_acquire(dst, val),
            Release => intrinsics::atomic_xsub_release(dst, val),
            AcqRel => intrinsics::atomic_xsub_acqrel(dst, val),
            SeqCst => intrinsics::atomic_xsub_seqcst(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_compare_exchange<T: Copy>(
    dst: *mut T,
    old: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
    // SAFETY: the caller must uphold the safety contract for `atomic_compare_exchange`.
    let (val, ok) = unsafe {
        match (success, failure) {
            (Relaxed, Relaxed) => intrinsics::atomic_cxchg_relaxed_relaxed(dst, old, new),
            (Relaxed, Acquire) => intrinsics::atomic_cxchg_relaxed_acquire(dst, old, new),
            (Relaxed, SeqCst) => intrinsics::atomic_cxchg_relaxed_seqcst(dst, old, new),
            (Acquire, Relaxed) => intrinsics::atomic_cxchg_acquire_relaxed(dst, old, new),
            (Acquire, Acquire) => intrinsics::atomic_cxchg_acquire_acquire(dst, old, new),
            (Acquire, SeqCst) => intrinsics::atomic_cxchg_acquire_seqcst(dst, old, new),
            (Release, Relaxed) => intrinsics::atomic_cxchg_release_relaxed(dst, old, new),
            (Release, Acquire) => intrinsics::atomic_cxchg_release_acquire(dst, old, new),
            (Release, SeqCst) => intrinsics::atomic_cxchg_release_seqcst(dst, old, new),
            (AcqRel, Relaxed) => intrinsics::atomic_cxchg_acqrel_relaxed(dst, old, new),
            (AcqRel, Acquire) => intrinsics::atomic_cxchg_acqrel_acquire(dst, old, new),
            (AcqRel, SeqCst) => intrinsics::atomic_cxchg_acqrel_seqcst(dst, old, new),
            (SeqCst, Relaxed) => intrinsics::atomic_cxchg_seqcst_relaxed(dst, old, new),
            (SeqCst, Acquire) => intrinsics::atomic_cxchg_seqcst_acquire(dst, old, new),
            (SeqCst, SeqCst) => intrinsics::atomic_cxchg_seqcst_seqcst(dst, old, new),
            (_, AcqRel) => panic!("there is no such thing as an acquire-release failure ordering"),
            (_, Release) => panic!("there is no such thing as a release failure ordering"),
        }
    };
    if ok {
        Ok(val)
    } else {
        Err(val)
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_compare_exchange_weak<T: Copy>(
    dst: *mut T,
    old: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
    // SAFETY: the caller must uphold the safety contract for `atomic_compare_exchange_weak`.
    let (val, ok) = unsafe {
        match (success, failure) {
            (Relaxed, Relaxed) => intrinsics::atomic_cxchgweak_relaxed_relaxed(dst, old, new),
            (Relaxed, Acquire) => intrinsics::atomic_cxchgweak_relaxed_acquire(dst, old, new),
            (Relaxed, SeqCst) => intrinsics::atomic_cxchgweak_relaxed_seqcst(dst, old, new),
            (Acquire, Relaxed) => intrinsics::atomic_cxchgweak_acquire_relaxed(dst, old, new),
            (Acquire, Acquire) => intrinsics::atomic_cxchgweak_acquire_acquire(dst, old, new),
            (Acquire, SeqCst) => intrinsics::atomic_cxchgweak_acquire_seqcst(dst, old, new),
            (Release, Relaxed) => intrinsics::atomic_cxchgweak_release_relaxed(dst, old, new),
            (Release, Acquire) => intrinsics::atomic_cxchgweak_release_acquire(dst, old, new),
            (Release, SeqCst) => intrinsics::atomic_cxchgweak_release_seqcst(dst, old, new),
            (AcqRel, Relaxed) => intrinsics::atomic_cxchgweak_acqrel_relaxed(dst, old, new),
            (AcqRel, Acquire) => intrinsics::atomic_cxchgweak_acqrel_acquire(dst, old, new),
            (AcqRel, SeqCst) => intrinsics::atomic_cxchgweak_acqrel_seqcst(dst, old, new),
            (SeqCst, Relaxed) => intrinsics::atomic_cxchgweak_seqcst_relaxed(dst, old, new),
            (SeqCst, Acquire) => intrinsics::atomic_cxchgweak_seqcst_acquire(dst, old, new),
            (SeqCst, SeqCst) => intrinsics::atomic_cxchgweak_seqcst_seqcst(dst, old, new),
            (_, AcqRel) => panic!("there is no such thing as an acquire-release failure ordering"),
            (_, Release) => panic!("there is no such thing as a release failure ordering"),
        }
    };
    if ok {
        Ok(val)
    } else {
        Err(val)
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_and<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_and`
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_and_relaxed(dst, val),
            Acquire => intrinsics::atomic_and_acquire(dst, val),
            Release => intrinsics::atomic_and_release(dst, val),
            AcqRel => intrinsics::atomic_and_acqrel(dst, val),
            SeqCst => intrinsics::atomic_and_seqcst(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_nand<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_nand`
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_nand_relaxed(dst, val),
            Acquire => intrinsics::atomic_nand_acquire(dst, val),
            Release => intrinsics::atomic_nand_release(dst, val),
            AcqRel => intrinsics::atomic_nand_acqrel(dst, val),
            SeqCst => intrinsics::atomic_nand_seqcst(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_or<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_or`
    unsafe {
        match order {
            SeqCst => intrinsics::atomic_or_seqcst(dst, val),
            Acquire => intrinsics::atomic_or_acquire(dst, val),
            Release => intrinsics::atomic_or_release(dst, val),
            AcqRel => intrinsics::atomic_or_acqrel(dst, val),
            Relaxed => intrinsics::atomic_or_relaxed(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_xor<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_xor`
    unsafe {
        match order {
            SeqCst => intrinsics::atomic_xor_seqcst(dst, val),
            Acquire => intrinsics::atomic_xor_acquire(dst, val),
            Release => intrinsics::atomic_xor_release(dst, val),
            AcqRel => intrinsics::atomic_xor_acqrel(dst, val),
            Relaxed => intrinsics::atomic_xor_relaxed(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_max<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_max`
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_max_relaxed(dst, val),
            Acquire => intrinsics::atomic_max_acquire(dst, val),
            Release => intrinsics::atomic_max_release(dst, val),
            AcqRel => intrinsics::atomic_max_acqrel(dst, val),
            SeqCst => intrinsics::atomic_max_seqcst(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_min<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_min`
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_min_relaxed(dst, val),
            Acquire => intrinsics::atomic_min_acquire(dst, val),
            Release => intrinsics::atomic_min_release(dst, val),
            AcqRel => intrinsics::atomic_min_acqrel(dst, val),
            SeqCst => intrinsics::atomic_min_seqcst(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_umax<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_umax`
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_umax_relaxed(dst, val),
            Acquire => intrinsics::atomic_umax_acquire(dst, val),
            Release => intrinsics::atomic_umax_release(dst, val),
            AcqRel => intrinsics::atomic_umax_acqrel(dst, val),
            SeqCst => intrinsics::atomic_umax_seqcst(dst, val),
        }
    }
}

#[inline]
#[cfg(target_has_atomic)]
pub(crate) unsafe fn atomic_umin<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    // SAFETY: the caller must uphold the safety contract for `atomic_umin`
    unsafe {
        match order {
            Relaxed => intrinsics::atomic_umin_relaxed(dst, val),
            Acquire => intrinsics::atomic_umin_acquire(dst, val),
            Release => intrinsics::atomic_umin_release(dst, val),
            AcqRel => intrinsics::atomic_umin_acqrel(dst, val),
            SeqCst => intrinsics::atomic_umin_seqcst(dst, val),
        }
    }
}
