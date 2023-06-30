use alloc::collections::BTreeMap;
use core::mem;
use core::task::{Poll, Waker};

pub struct Requests<K, T>(BTreeMap<K, RequestStatus<T>>);

impl<K: Ord, T> Requests<K, T> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add(&mut self, request: K) {
        assert!(self
            .0
            .insert(request, RequestStatus::Incomplete(None))
            .is_none());
    }

    pub fn mark_complete(&mut self, request: &K, value: T) {
        self.0.get_mut(request).unwrap().mark_complete(value);
    }

    pub fn poll(&mut self, request: &K, waker: &Waker) -> Poll<T> {
        let (request, status) = self.0.remove_entry(request).unwrap();
        match status {
            RequestStatus::Complete(value) => Poll::Ready(value),
            RequestStatus::Incomplete(_) => {
                self.0
                    .insert(request, RequestStatus::Incomplete(Some(waker.clone())));
                Poll::Pending
            }
        }
    }
}

enum RequestStatus<T> {
    Complete(T),
    Incomplete(Option<Waker>),
}

impl<T> RequestStatus<T> {
    fn mark_complete(&mut self, value: T) {
        match mem::replace(self, Self::Complete(value)) {
            Self::Complete(_) => {
                panic!();
            }
            Self::Incomplete(maybe_waker) => {
                maybe_waker.map(Waker::wake);
            }
        }
    }
}
