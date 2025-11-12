//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::time::Duration;

use rustls::pki_types::UnixTime;
use rustls::time_provider::TimeProvider;

use sel4_async_time::Instant;

pub struct TimeProviderImpl<F> {
    start_global: UnixTime,
    start_local: Instant,
    now_fn: F,
}

impl<F> fmt::Debug for TimeProviderImpl<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeProviderImpl").finish()
    }
}

impl<F: Send + Sync + Fn() -> Instant> TimeProviderImpl<F> {
    pub fn new(now_global: UnixTime, now_fn: F) -> Self {
        let start_local = now_fn();
        Self {
            start_global: now_global,
            start_local,
            now_fn,
        }
    }
}

impl<F: Send + Sync + Fn() -> Instant> TimeProvider for TimeProviderImpl<F> {
    fn current_time(&self) -> Option<UnixTime> {
        Some(UnixTime::since_unix_epoch(
            Duration::from_secs(self.start_global.as_secs()) + ((self.now_fn)() - self.start_local),
        ))
    }
}
