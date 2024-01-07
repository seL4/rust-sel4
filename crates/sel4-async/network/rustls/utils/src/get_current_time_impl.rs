//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::time::Duration;

use rustls::pki_types::UnixTime;
use rustls::time_provider::GetCurrentTime;

use sel4_async_time::Instant;

pub struct GetCurrentTimeImpl<F> {
    start_global: UnixTime,
    start_local: Instant,
    now_fn: F,
}

impl<F> fmt::Debug for GetCurrentTimeImpl<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetCurrentTimeImpl").finish()
    }
}

impl<F: Send + Sync + Fn() -> Instant> GetCurrentTimeImpl<F> {
    pub fn new(now_global: UnixTime, now_fn: F) -> Self {
        let start_local = now_fn();
        Self {
            start_global: now_global,
            start_local,
            now_fn,
        }
    }
}

impl<F: Send + Sync + Fn() -> Instant> GetCurrentTime for GetCurrentTimeImpl<F> {
    fn get_current_time(&self) -> Option<UnixTime> {
        Some(UnixTime::since_unix_epoch(
            Duration::from_secs(self.start_global.as_secs()) + ((self.now_fn)() - self.start_local),
        ))
    }
}
