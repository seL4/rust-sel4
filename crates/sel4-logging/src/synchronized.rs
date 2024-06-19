//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use lock_api::{Mutex, RawMutex};
use log::{Log, Metadata, Record};

pub struct SynchronizedLogger<R, T>(Mutex<R, T>);

impl<R: RawMutex, T> SynchronizedLogger<R, T> {
    pub const fn new(inner: T) -> Self {
        Self(Mutex::new(inner))
    }
}

impl<R, T> SynchronizedLogger<R, T> {
    pub const fn from_raw(raw_mutex: R, inner: T) -> Self {
        Self(Mutex::from_raw(raw_mutex, inner))
    }

    pub fn into_inner(self) -> Mutex<R, T> {
        self.0
    }

    pub fn inner(&self) -> &Mutex<R, T> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut Mutex<R, T> {
        &mut self.0
    }
}

impl<R: RawMutex + Send + Sync, T: Log> Log for SynchronizedLogger<R, T> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner().lock().enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.inner().lock().log(record)
    }

    fn flush(&self) {
        self.inner().lock().flush()
    }
}
