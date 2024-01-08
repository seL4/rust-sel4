//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use pin_project::pin_project;

mod instant;
mod sub_key;
mod timer_queue;

use sub_key::SubKey;
use timer_queue::{Key, TimerQueue};

pub use instant::Instant;

#[derive(Clone)]
pub struct TimerManager {
    shared: Rc<RefCell<TimerManagerShared>>,
}

struct TimerManagerShared {
    pending: TimerQueue<Instant, usize, Rc<RefCell<TimerShared>>>,
}

struct TimerShared {
    expired: bool,
    waker: Option<Waker>,
}

impl TimerShared {
    fn mark_expired(&mut self) {
        assert!(!self.expired);
        self.expired = true;
        if let Some(waker) = self.waker.take() {
            waker.wake();
        };
    }
}

impl TimerManagerShared {
    fn new() -> Self {
        Self {
            pending: TimerQueue::new(),
        }
    }

    fn poll(&mut self, timestamp: Instant) -> bool {
        let mut activity = false;
        for expired in self.pending.iter_expired(timestamp) {
            expired.value().borrow_mut().mark_expired();
            activity = true;
        }
        activity
    }

    fn poll_at(&mut self) -> Option<Instant> {
        self.pending.peek_next_absolute_expiry().copied()
    }
}

impl TimerManager {
    #![allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            shared: Rc::new(RefCell::new(TimerManagerShared::new())),
        }
    }

    fn shared(&self) -> &RefCell<TimerManagerShared> {
        &self.shared
    }

    pub fn poll(&self, timestamp: Instant) -> bool {
        self.shared().borrow_mut().poll(timestamp)
    }

    pub fn poll_at(&self) -> Option<Instant> {
        self.shared().borrow_mut().poll_at()
    }

    pub fn sleep_until(&self, absolute_expiry: Instant) -> Sleep {
        let timer_shared = Rc::new(RefCell::new(TimerShared {
            expired: false,
            waker: None,
        }));
        let timer_key = self
            .shared()
            .borrow_mut()
            .pending
            .insert(absolute_expiry, timer_shared.clone());
        Sleep {
            timer_manager: self.clone(),
            timer_key,
            timer_shared,
        }
    }

    pub fn timeout_at<F: Future>(&self, absolute_deadline: Instant, future: F) -> Timeout<F> {
        Timeout {
            value: future,
            sleep: self.sleep_until(absolute_deadline),
        }
    }
}

pub struct Sleep {
    timer_manager: TimerManager,
    timer_key: Key<Instant, usize>,
    timer_shared: Rc<RefCell<TimerShared>>,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut timer_shared = self.timer_shared.borrow_mut();
        if timer_shared.expired {
            Poll::Ready(())
        } else {
            timer_shared.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        if !self.timer_shared.borrow().expired {
            self.timer_manager
                .shared()
                .borrow_mut()
                .pending
                .remove(&self.timer_key);
        }
    }
}

#[pin_project]
pub struct Timeout<F> {
    #[pin]
    value: F,
    #[pin]
    sleep: Sleep,
}

impl<F: Future> Future for Timeout<F> {
    type Output = Result<F::Output, Elapsed>;

    #[allow(clippy::redundant_pattern_matching)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Poll::Ready(v) = this.value.poll(cx) {
            return Poll::Ready(Ok(v));
        }
        if let Poll::Ready(_) = this.sleep.poll(cx) {
            return Poll::Ready(Err(Elapsed::new()));
        }
        Poll::Pending
    }
}

/// Error returned by `Timeout`.
///
/// This error is returned when a timeout expires before the function was able to finish.
#[derive(Debug, PartialEq, Eq)]
pub struct Elapsed(());

impl Elapsed {
    fn new() -> Self {
        Elapsed(())
    }
}

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "deadline has elapsed")
    }
}
