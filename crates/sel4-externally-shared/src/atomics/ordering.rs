//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::sync::atomic::Ordering;

pub(crate) enum OrderingExhaustive {
    Relaxed,
    Release,
    Acquire,
    AcqRel,
    SeqCst,
}

impl From<OrderingExhaustive> for Ordering {
    fn from(order: OrderingExhaustive) -> Self {
        match order {
            OrderingExhaustive::Relaxed => Self::Relaxed,
            OrderingExhaustive::Release => Self::Release,
            OrderingExhaustive::Acquire => Self::Acquire,
            OrderingExhaustive::AcqRel => Self::AcqRel,
            OrderingExhaustive::SeqCst => Self::SeqCst,
        }
    }
}

impl From<Ordering> for OrderingExhaustive {
    fn from(order: Ordering) -> Self {
        match order {
            Ordering::Relaxed => Self::Relaxed,
            Ordering::Release => Self::Release,
            Ordering::Acquire => Self::Acquire,
            Ordering::AcqRel => Self::AcqRel,
            Ordering::SeqCst => Self::SeqCst,
            _ => panic!(),
        }
    }
}
