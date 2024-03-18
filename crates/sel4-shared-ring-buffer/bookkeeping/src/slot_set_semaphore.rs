//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::rc::Rc;
use core::array;
use core::cell::Cell;
use core::future::Future;
use core::mem;

use crate::slot_count_tracker::{SlotCountTracker, SlotCountTrackerError};

pub trait SlotSemaphore {
    fn new(count: usize) -> Self;

    fn try_take(&self, n: usize) -> Result<bool, SlotSemaphoreClosedError>;

    fn give(&self, n: usize);

    fn close(&self);
}

pub trait AsyncSlotSemaphore: SlotSemaphore {
    #[allow(clippy::needless_lifetimes)]
    fn take<'a>(
        &'a self,
        n: usize,
    ) -> impl Future<Output = Result<(), SlotSemaphoreClosedError>> + 'a;
}

pub struct SlotSetSemaphore<T, const N: usize> {
    per_pool_slot_count_trackers: [SlotCountTracker; N],
    handle: SlotSetSemaphoreHandle<T, N>,
}

#[derive(Clone)]
pub struct SlotSetSemaphoreHandle<T, const N: usize> {
    per_pool_semaphores: [T; N],
}

pub struct SlotSetReservation<'a, T: SlotSemaphore, const N: usize> {
    handle: &'a SlotSetSemaphoreHandle<T, N>,
    n: usize,
}

impl<T: SlotSemaphore, const N: usize> SlotSetSemaphore<T, N> {
    pub fn new(slot_pool_capacities: [usize; N]) -> Self {
        Self {
            per_pool_slot_count_trackers: array::from_fn(|i| {
                SlotCountTracker::new(slot_pool_capacities[i])
            }),
            handle: SlotSetSemaphoreHandle {
                per_pool_semaphores: array::from_fn(|i| T::new(slot_pool_capacities[i])),
            },
        }
    }

    pub fn close(&self) {
        for sem in &self.handle.per_pool_semaphores {
            sem.close();
        }
    }

    pub fn handle(&self) -> &SlotSetSemaphoreHandle<T, N> {
        &self.handle
    }

    pub fn consume(
        &mut self,
        reservation: &mut SlotSetReservation<'_, T, N>,
        n: usize,
    ) -> Result<(), Error> {
        reservation.reduce_count(n)?;
        for tracker in &mut self.per_pool_slot_count_trackers {
            tracker.report_occupied(n)?;
        }
        Ok(())
    }

    pub fn report_current_num_free_slots(
        &mut self,
        slot_pool_index: usize,
        current_num_free_slots: usize,
    ) -> Result<(), SlotCountTrackerError> {
        self.handle.per_pool_semaphores[slot_pool_index].give(
            self.per_pool_slot_count_trackers[slot_pool_index]
                .redeem_newly_free(current_num_free_slots)?,
        );
        Ok(())
    }
}

impl<T: SlotSemaphore, const N: usize> SlotSetSemaphoreHandle<T, N> {
    pub fn try_reserve(
        &self,
        n: usize,
    ) -> Result<Option<SlotSetReservation<'_, T, N>>, SlotSemaphoreClosedError> {
        let mut tmp_permits: [Option<TemporaryPermit<'_, T>>; N] = array::from_fn(|_| None);
        for (i, sem) in self.per_pool_semaphores.iter().enumerate() {
            if sem.try_take(n)? {
                tmp_permits[i] = Some(TemporaryPermit::new(sem, n));
            } else {
                return Ok(None);
            }
        }

        mem::forget(tmp_permits);

        Ok(Some(self.reservation(n)))
    }

    fn reservation(&self, n: usize) -> SlotSetReservation<'_, T, N> {
        SlotSetReservation { handle: self, n }
    }
}

impl<T: AsyncSlotSemaphore, const N: usize> SlotSetSemaphoreHandle<T, N> {
    pub async fn reserve(
        &self,
        n: usize,
    ) -> Result<SlotSetReservation<'_, T, N>, SlotSemaphoreClosedError> {
        let mut tmp_permits: [Option<TemporaryPermit<'_, T>>; N] = array::from_fn(|_| None);
        for (i, sem) in self.per_pool_semaphores.iter().enumerate() {
            sem.take(n).await?;
            tmp_permits[i] = Some(TemporaryPermit::new(sem, n));
        }

        mem::forget(tmp_permits);

        Ok(self.reservation(n))
    }
}

struct TemporaryPermit<'a, T: SlotSemaphore> {
    sem: &'a T,
    n: usize,
}

impl<'a, T: SlotSemaphore> TemporaryPermit<'a, T> {
    fn new(sem: &'a T, n: usize) -> Self {
        Self { sem, n }
    }
}

impl<'a, T: SlotSemaphore> Drop for TemporaryPermit<'a, T> {
    fn drop(&mut self) {
        self.sem.give(self.n);
    }
}

impl<'a, T: SlotSemaphore, const N: usize> SlotSetReservation<'a, T, N> {
    pub fn count(&self) -> usize {
        self.n
    }

    fn reduce_count(&mut self, n: usize) -> Result<(), SlotReservationExhaustedError> {
        if n > self.n {
            return Err(SlotReservationExhaustedError::new());
        }
        self.n -= n;
        Ok(())
    }

    pub fn split(&mut self, split_off: usize) -> Result<Self, SlotReservationExhaustedError> {
        self.reduce_count(split_off)?;
        Ok(Self {
            handle: self.handle,
            n: split_off,
        })
    }

    pub fn merge(&mut self, other: Self) {
        self.n = self.n.checked_add(other.n).unwrap();
    }
}

impl<'a, T: SlotSemaphore, const N: usize> Drop for SlotSetReservation<'a, T, N> {
    fn drop(&mut self) {
        for sem in &self.handle.per_pool_semaphores {
            sem.give(self.n);
        }
    }
}

#[derive(Debug)]
pub enum Error {
    SlotReservationExhausted,
    SlotCountTrackingError,
}

impl From<SlotReservationExhaustedError> for Error {
    fn from(_err: SlotReservationExhaustedError) -> Self {
        Self::SlotReservationExhausted
    }
}

impl From<SlotCountTrackerError> for Error {
    fn from(_err: SlotCountTrackerError) -> Self {
        Self::SlotCountTrackingError
    }
}

#[derive(Debug)]
pub struct SlotSemaphoreClosedError(());

impl SlotSemaphoreClosedError {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}

#[derive(Debug)]
pub struct SlotReservationExhaustedError(());

impl SlotReservationExhaustedError {
    #![allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}

pub struct DummySlotSemaphore {
    permits: Cell<Option<usize>>,
}

impl SlotSemaphore for Rc<DummySlotSemaphore> {
    fn new(count: usize) -> Self {
        Rc::new(DummySlotSemaphore {
            permits: Cell::new(Some(count)),
        })
    }

    fn try_take(&self, n: usize) -> Result<bool, SlotSemaphoreClosedError> {
        match self.permits.get() {
            Some(permits) => Ok(match n.checked_sub(permits) {
                Some(new_permits) => {
                    self.permits.set(Some(new_permits));
                    true
                }
                None => false,
            }),
            None => Err(SlotSemaphoreClosedError::new()),
        }
    }

    fn give(&self, n: usize) {
        if let Some(permits) = self.permits.get() {
            self.permits.set(Some(permits.checked_add(n).unwrap()));
        }
    }

    fn close(&self) {
        self.permits.set(None);
    }
}

#[cfg(feature = "async-unsync")]
mod async_unsync_impl {
    use async_unsync::semaphore::{Semaphore, TryAcquireError};

    use super::*;

    impl SlotSemaphore for Rc<Semaphore> {
        fn new(count: usize) -> Self {
            Rc::new(Semaphore::new(count))
        }

        fn try_take(&self, n: usize) -> Result<bool, SlotSemaphoreClosedError> {
            match self.try_acquire_many(n) {
                Ok(permit) => {
                    permit.forget();
                    Ok(true)
                }
                Err(TryAcquireError::NoPermits) => Ok(false),
                Err(TryAcquireError::Closed) => Err(SlotSemaphoreClosedError::new()),
            }
        }

        fn give(&self, n: usize) {
            self.add_permits(n)
        }

        fn close(&self) {
            Semaphore::close(self);
        }
    }

    impl AsyncSlotSemaphore for Rc<Semaphore> {
        async fn take(&self, n: usize) -> Result<(), SlotSemaphoreClosedError> {
            self.acquire_many(n)
                .await
                .map_err(|_| SlotSemaphoreClosedError::new())?
                .forget();
            Ok(())
        }
    }
}
