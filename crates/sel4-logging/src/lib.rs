//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::fmt::{self, Write};

use log::{Log, Metadata, Record, SetLoggerError};

pub use log::{self, LevelFilter};

mod synchronized;

pub use synchronized::SynchronizedLogger;

pub struct Logger {
    pub level_filter: LevelFilter,
    pub filter: fn(&Metadata) -> bool,
    pub fmt: FmtRecordFn,
    pub write: fn(&str),
    pub flush: fn(),
}

pub type FmtRecordFn = fn(&Record, &mut fmt::Formatter) -> fmt::Result;

pub const FMT_RECORD_DEFAULT: FmtRecordFn = fmt_with_module;

impl Logger {
    pub const fn const_default() -> Self {
        Self {
            level_filter: LevelFilter::Warn,
            filter: |_| true,
            fmt: FMT_RECORD_DEFAULT,
            write: |_| (),
            flush: || (),
        }
    }

    pub fn level_filter(&self) -> LevelFilter {
        self.level_filter
    }

    pub fn set_max_level(&self) {
        log::set_max_level(self.level_filter());
    }

    pub fn set(&'static self) -> Result<(), SetLoggerError> {
        self.set_max_level();
        log::set_logger(self)?;
        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter && (self.filter)(metadata)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut writer = WriteWrapper(self.write);
            let wrapped = DisplayWrapper {
                fmt: self.fmt,
                record,
            };
            writeln!(writer, "{wrapped}").unwrap()
        }
    }

    fn flush(&self) {
        (self.flush)()
    }
}

//

struct WriteWrapper(fn(&str));

impl fmt::Write for WriteWrapper {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        (self.0)(s);
        Ok(())
    }
}

struct DisplayWrapper<'a> {
    fmt: FmtRecordFn,
    record: &'a Record<'a>,
}

impl<'a> fmt::Display for DisplayWrapper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.fmt)(self.record, f)
    }
}

//

pub struct LoggerBuilder(Logger);

impl LoggerBuilder {
    pub const fn const_default() -> Self {
        Self(Logger::const_default())
    }

    pub const fn build(self) -> Logger {
        self.0
    }

    pub const fn level_filter(mut self, level_filter: LevelFilter) -> Self {
        self.0.level_filter = level_filter;
        self
    }

    pub const fn filter(mut self, filter: fn(&Metadata) -> bool) -> Self {
        self.0.filter = filter;
        self
    }

    pub const fn fmt(mut self, fmt: FmtRecordFn) -> Self {
        self.0.fmt = fmt;
        self
    }

    pub const fn write(mut self, write: fn(&str)) -> Self {
        self.0.write = write;
        self
    }

    pub const fn flush(mut self, flush: fn()) -> Self {
        self.0.flush = flush;
        self
    }
}

//

pub fn fmt_with_module(record: &Record, f: &mut fmt::Formatter) -> fmt::Result {
    let target = if !record.target().is_empty() {
        record.target()
    } else {
        record.module_path().unwrap_or_default()
    };
    write!(f, "{:<5} [{}] {}", record.level(), target, record.args())
}

pub fn fmt_with_line(record: &Record, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:<5} [", record.level())?;
    if let Some(file) = record.file() {
        write!(f, "{file}")?;
    } else if let Some(file) = record.file_static() {
        write!(f, "{file}")?;
    } else {
        write!(f, "(?)")?;
    }
    write!(f, ":")?;
    if let Some(line) = record.line() {
        write!(f, "{line}")?;
    } else {
        write!(f, "(?)")?;
    }
    write!(f, "] {}", record.args())
}
