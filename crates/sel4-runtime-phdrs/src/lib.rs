#![no_std]
#![feature(cfg_target_thread_local)]

pub mod elf;

#[cfg(all(feature = "tls", target_thread_local))]
mod tls;

#[cfg(feature = "unwinding")]
pub mod unwinding;

#[cfg(feature = "embedded-phdrs")]
mod embedded;

#[cfg(feature = "embedded-phdrs")]
pub use embedded::EmbeddedProgramHeaders;

use elf::ProgramHeader;

pub trait InnerProgramHeadersFinder {
    fn find_phdrs(&self) -> &[ProgramHeader];
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProgramHeadersFinder<T>(T);

impl<T> ProgramHeadersFinder<T> {
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T: InnerProgramHeadersFinder> ProgramHeadersFinder<T> {
    pub fn find_phdrs(&self) -> &[ProgramHeader] {
        self.0.find_phdrs()
    }
}
