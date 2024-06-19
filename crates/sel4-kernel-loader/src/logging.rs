//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use spin::Mutex;

use sel4_logging::{LevelFilter, Logger, LoggerBuilder, SynchronizedLogger};

use crate::fmt::debug_print;

const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

static LOGGER: SynchronizedLogger<Mutex<()>, Logger> = SynchronizedLogger::new(
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
    log::set_max_level(LOGGER.inner().lock().level_filter);
    log::set_logger(&LOGGER).unwrap();
}
