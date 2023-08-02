#![no_std]
#![feature(btree_cursors)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::ops::Bound;
use core::task::Poll;
use core::task::Waker;

use futures::prelude::*;
use smoltcp::time::{Duration, Instant};

#[derive(Clone)]
pub struct SharedTimers {
    inner: Rc<RefCell<SharedTimersInner>>,
}

pub struct SharedTimersInner {
    pending: BTreeMap<Instant, Vec<Waker>>,
    now: Instant,
}

impl SharedTimers {
    pub fn new(now: Instant) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SharedTimersInner::new(now))),
        }
    }

    pub fn inner(&self) -> &Rc<RefCell<SharedTimersInner>> {
        &self.inner
    }

    async fn sleep_absolute(&self, until: Instant) {
        future::poll_fn(|cx| {
            let mut inner = self.inner().borrow_mut();
            if inner.now() < &until {
                inner.set_timer(until, cx.waker());
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
    }

    pub async fn sleep(&self, d: Duration) {
        let now = self.inner().borrow().now().clone();
        self.sleep_absolute(now + d).await;
    }
}

impl SharedTimersInner {
    fn new(now: Instant) -> Self {
        Self {
            pending: BTreeMap::new(),
            now,
        }
    }

    fn now(&self) -> &Instant {
        &self.now
    }

    pub fn poll(&mut self, timestamp: Instant) -> bool {
        self.now = timestamp;
        let mut cursor = self.pending.upper_bound_mut(Bound::Included(&timestamp));
        let mut activity = false;
        while cursor.remove_current_and_move_back().is_some() {
            activity = true;
        }
        activity
    }

    pub fn poll_at(&mut self, timestamp: Instant) -> Option<Instant> {
        self.now = timestamp;
        self.pending.first_entry().map(|entry| entry.key().clone())
    }

    pub fn poll_delay(&mut self, timestamp: Instant) -> Option<Duration> {
        self.poll_at(timestamp)
            .map(|deadline| deadline.max(timestamp) - timestamp)
    }

    fn set_timer(&mut self, expiry: Instant, waker: &Waker) {
        self.pending.entry(expiry).or_default().push(waker.clone());
    }
}
