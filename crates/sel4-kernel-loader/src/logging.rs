//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use log::{Log, Metadata, Record};
use spin::Mutex;

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};

use crate::fmt::debug_print;

const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

static LOGGER: SynchronizedLogger<Logger> = SynchronizedLogger::new(
    LoggerBuilder::const_default()
        .level_filter(LOG_LEVEL)
        .write(|s| debug_print!("{}", s))
        .fmt(|record, f| {
            write!(
                f,
                "seL4 kernel loader | {:<5}  {}",
                record.level(),
                record.args()
            )
        })
        .build(),
);

pub(crate) fn set_logger() {
    log::set_max_level(LOGGER.0.lock().level_filter);
    log::set_logger(&LOGGER).unwrap();
}

struct SynchronizedLogger<T>(Mutex<T>);

impl<T> SynchronizedLogger<T> {
    const fn new(inner: T) -> Self {
        Self(Mutex::new(inner))
    }
}

impl<T: Log> Log for SynchronizedLogger<T> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0.lock().enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.0.lock().log(record)
    }

    fn flush(&self) {
        self.0.lock().flush()
    }
}
