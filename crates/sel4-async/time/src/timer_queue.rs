//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::collections::btree_map::{BTreeMap, Entry};

use crate::SubKey;

// We opt for a simple B-tree-based implementation rather than implementing a timer wheel. This is
// good enough for now. Note that this approach is also good enough for `async-std`. If we ever do
// actually need the scalability of something like a timer wheel, `tokio`'s implementation would be
// a good place to start.

// TODO
// Add feature like `tokio::time::Interval`

// NOTE
// Once #![feature(btree_cursors)] stabilizes, revert back to using it for a simpler, more
// lightweight, and more efficient (on the small scale) implementation. See git history for such an
// implementation.

pub struct TimerQueue<T, U, V> {
    pending: BTreeMap<T, BTreeMap<U, V>>,
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
        let (absolute_expiry, _value) = self.pending.first_key_value()?;
        Some(absolute_expiry)
    }

    pub fn insert(&mut self, absolute_expiry: T, value: V) -> Key<T, U> {
        let sub_key = match self.pending.entry(absolute_expiry.clone()) {
            Entry::Vacant(entry) => {
                let sub_key = <U as SubKey>::min();
                entry.insert(BTreeMap::new()).insert(sub_key.clone(), value);
                sub_key
            }
            Entry::Occupied(mut entry) => {
                let sub_map = entry.get_mut();
                let sub_key = sub_map
                    .last_entry()
                    .unwrap()
                    .key()
                    .succ()
                    .expect("too many timers for one instant");
                sub_map.insert(sub_key.clone(), value);
                sub_key
            }
        };

        Key {
            absolute_expiry,
            sub_key,
        }
    }

    pub fn try_remove(&mut self, key: &Key<T, U>) -> Option<Expired<T, U, V>> {
        match self
            .pending
            .entry(key.absolute_expiry().clone() /* HACK silly clone */)
        {
            Entry::Vacant(_) => None,
            Entry::Occupied(mut entry) => {
                let sub_map = entry.get_mut();
                let (sub_key, value) = sub_map.remove_entry(&key.sub_key)?;
                if sub_map.is_empty() {
                    entry.remove();
                }
                Some(Expired {
                    key: Key {
                        absolute_expiry: key.absolute_expiry().clone(),
                        sub_key,
                    },
                    value,
                })
            }
        }
    }

    pub fn remove(&mut self, key: &Key<T, U>) -> Expired<T, U, V> {
        self.try_remove(key).unwrap()
    }

    pub fn iter_expired(&mut self, now: T) -> IterExpired<'_, T, U, V> {
        IterExpired { now, queue: self }
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
    queue: &'a mut TimerQueue<T, U, V>,
}

impl<T: Ord + Clone, U: Ord, V> Iterator for IterExpired<'_, T, U, V> {
    type Item = Expired<T, U, V>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = self.queue.pending.first_entry()?;
        if entry.key() > &self.now {
            return None;
        }
        let sub_map = entry.get_mut();
        let (sub_key, value) = sub_map.pop_first().unwrap();
        let absolute_expiry = if sub_map.is_empty() {
            entry.remove_entry().0
        } else {
            entry.key().clone()
        };
        Some(Expired {
            key: Key {
                absolute_expiry,
                sub_key,
            },
            value,
        })
    }
}
