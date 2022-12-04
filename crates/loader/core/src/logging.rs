use core::fmt;

use spin::Mutex;

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::fmt::debug_println;

pub struct Logger {
    level_filter: LevelFilter,
    mutex: Mutex<()>,
}

impl Logger {
    pub const fn new(level_filter: LevelFilter) -> Self {
        Self {
            level_filter,
            mutex: Mutex::new(()),
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let guard = self.mutex.lock();
            debug_println!("{}", RecordDisplay(record));
            drop(guard);
        }
    }

    fn flush(&self) {}
}

impl Logger {
    pub fn set(&'static self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.level_filter);
        log::set_logger(self)?;
        Ok(())
    }
}

struct RecordDisplay<'a>(&'a Record<'a>);

impl<'a> fmt::Display for RecordDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let record = self.0;
        write!(f, "seL4 loader [{:<5}] {}", record.level(), record.args())
    }
}
