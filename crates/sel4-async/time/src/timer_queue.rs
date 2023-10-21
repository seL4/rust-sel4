//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::collections::btree_map::{BTreeMap, CursorMut};
use core::ops::Bound;

use crate::SubKey;

// We opt for a simple B-tree-based implementation rather than implementing a timer wheel. This is
// good enough for now. Note that this approach is also good enough for `async-std`. If we ever do
// actually need the scalability of something like a timer wheel, `tokio`'s implementation would be
// a good place to start.

// TODO: Add feature like `tokio::time::Interval`

pub struct TimerQueue<T, U, V> {
    pending: BTreeMap<Key<T, U>, V>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key<T, U> {
    absolute_expiry: T,
    sub_key: U,
}

impl<T, U> Key<T, U> {
    pub fn absolute_expiry(&self) -> &T {
        &self.absolute_expiry
    }
}

impl<T: Ord + Clone, U: SubKey + Clone, V> TimerQueue<T, U, V> {
    pub fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
        }
    }

    pub fn peek_next_absolute_expiry(&self) -> Option<&T> {
        let (key, _value) = self.pending.first_key_value()?;
        Some(key.absolute_expiry())
    }

    pub fn insert(&mut self, absolute_expiry: T, value: V) -> Key<T, U> {
        let upper_bound = Key {
            absolute_expiry: absolute_expiry.clone(),
            sub_key: SubKey::max(),
        };
        let mut cursor = self.pending.upper_bound_mut(Bound::Included(&upper_bound));
        let sub_key = match cursor.key() {
            None => SubKey::min(),
            Some(prev_key) if prev_key.absolute_expiry() == &absolute_expiry => prev_key
                .sub_key
                .succ()
                .expect("too many timers for one instant"),
            Some(_) => SubKey::max(),
        };
        let key = Key {
            absolute_expiry,
            sub_key,
        };
        cursor.insert_after(key.clone(), value);
        key
    }

    pub fn try_remove(&mut self, key: &Key<T, U>) -> Option<Expired<T, U, V>> {
        let (key, value) = self.pending.remove_entry(key)?;
        Some(Expired { key, value })
    }

    pub fn remove(&mut self, key: &Key<T, U>) -> Expired<T, U, V> {
        self.try_remove(key).unwrap()
    }

    pub fn iter_expired(&mut self, now: T) -> IterExpired<'_, T, U, V> {
        IterExpired {
            now,
            cursor: self.pending.lower_bound_mut(Bound::Unbounded),
        }
    }
}

pub struct Expired<T, U, V> {
    #[allow(dead_code)]
    key: Key<T, U>,
    value: V,
}

impl<T, U, V> Expired<T, U, V> {
    #[allow(dead_code)]
    pub fn key(&self) -> &Key<T, U> {
        &self.key
    }

    #[allow(dead_code)]
    pub fn absolute_expiry(&self) -> &T {
        self.key().absolute_expiry()
    }

    pub fn value(&self) -> &V {
        &self.value
    }
}

pub struct IterExpired<'a, T, U, V> {
    now: T,
    cursor: CursorMut<'a, Key<T, U>, V>,
}

impl<'a, T: Ord, U: Ord, V> Iterator for IterExpired<'a, T, U, V> {
    type Item = Expired<T, U, V>;

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.cursor.key()?;
        if key.absolute_expiry() <= &self.now {
            let (key, value) = self.cursor.remove_current().unwrap();
            Some(Expired { key, value })
        } else {
            None
        }
    }
}
