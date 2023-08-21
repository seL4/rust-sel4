#![no_std]
#![feature(map_try_insert)]

extern crate alloc;

use alloc::collections::BTreeMap;
use core::mem;
use core::task::{Poll, Waker};

pub struct RequestStatuses<K, V, T>(BTreeMap<K, RequestEntry<V, T>>);

struct RequestEntry<V, T> {
    value: V,
    status: RequestStatus<T>,
}

enum RequestStatus<T> {
    Complete(T),
    Incomplete(Option<Waker>),
}

pub struct Completion<K, V, T> {
    pub key: K,
    pub value: V,
    pub complete: T,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Error {
    NotPresent,
    AlreadyPresent,
    AlreadyComplete,
}

impl<K: Ord, V, T> RequestStatuses<K, V, T> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add(&mut self, key: K, value: V) -> Result<(), Error> {
        self.0
            .try_insert(
                key,
                RequestEntry {
                    value,
                    status: RequestStatus::Incomplete(None),
                },
            )
            .map(|_| ())
            .map_err(|_| Error::AlreadyPresent)
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.0.get(key).map(|entry| &entry.value)
    }

    pub fn mark_complete(&mut self, key: &K, complete: T) -> Result<(), Error> {
        self.0
            .get_mut(key)
            .ok_or(Error::NotPresent)?
            .status
            .mark_complete(complete)
    }

    pub fn poll(&mut self, key: &K, waker: &Waker) -> Result<Poll<Completion<K, V, T>>, Error> {
        let (key, entry) = self.0.remove_entry(key).ok_or(Error::NotPresent)?;
        let value = entry.value;
        Ok(match entry.status {
            RequestStatus::Complete(complete) => Poll::Ready(Completion {
                key,
                value,
                complete,
            }),
            RequestStatus::Incomplete(_) => {
                self.0.insert(
                    key,
                    RequestEntry {
                        value,
                        status: RequestStatus::Incomplete(Some(waker.clone())),
                    },
                );
                Poll::Pending
            }
        })
    }
}

impl<T> RequestStatus<T> {
    fn mark_complete(&mut self, complete: T) -> Result<(), Error> {
        match mem::replace(self, Self::Complete(complete)) {
            Self::Complete(_) => Err(Error::AlreadyComplete),
            Self::Incomplete(maybe_waker) => {
                maybe_waker.map(Waker::wake);
                Ok(())
            }
        }
    }

    // fn set_waker(&mut self, waker: &Waker) -> Result<(), Error> {
    //     match mem::replace(self, Self::Incomplete(Some(waker.clone()))) {
    //         Self::Complete(_) => Err(Error::AlreadyComplete),
    //         _ => Ok(()),
    //     }
    // }
}
