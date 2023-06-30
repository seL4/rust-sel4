use alloc::collections::BTreeMap;
use core::mem;
use core::task::{Poll, Waker};

pub struct RequestStatuses<K, T>(BTreeMap<K, RequestStatus<T>>);

enum RequestStatus<T> {
    Complete(T),
    Incomplete(Option<Waker>),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Error {
    NotPresent,
    AlreadyPresent,
    AlreadyComplete,
}

impl<K: Ord, T> RequestStatuses<K, T> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add(&mut self, request: K) -> Result<(), Error> {
        self.0
            .try_insert(request, RequestStatus::Incomplete(None))
            .map(|_| ())
            .map_err(|_| Error::AlreadyPresent)
    }

    pub fn mark_complete(&mut self, request: &K, value: T) -> Result<(), Error> {
        self.0
            .get_mut(request)
            .ok_or(Error::NotPresent)?
            .mark_complete(value)
    }

    pub fn poll(&mut self, request: &K, waker: &Waker) -> Result<Poll<T>, Error> {
        let (request, status) = self.0.remove_entry(request).ok_or(Error::NotPresent)?;
        Ok(match status {
            RequestStatus::Complete(value) => Poll::Ready(value),
            RequestStatus::Incomplete(_) => {
                self.0
                    .insert(request, RequestStatus::Incomplete(Some(waker.clone())));
                Poll::Pending
            }
        })
    }
}

impl<T> RequestStatus<T> {
    fn mark_complete(&mut self, value: T) -> Result<(), Error> {
        match mem::replace(self, Self::Complete(value)) {
            Self::Complete(_) => Err(Error::AlreadyComplete),
            Self::Incomplete(maybe_waker) => {
                maybe_waker.map(Waker::wake);
                Ok(())
            }
        }
    }
}
