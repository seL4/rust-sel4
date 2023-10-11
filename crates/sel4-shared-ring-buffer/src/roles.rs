use crate::InitializationStrategy;

pub enum Provide {}
pub enum Use {}

pub trait RingBuffersRole: RingBuffersRoleSealed {
    type FreeRole: RingBufferRole;
    type UsedRole: RingBufferRole;

    const ROLE: RingBuffersRoleValue;

    fn default_initialization_strategy() -> InitializationStrategy;
}

impl RingBuffersRole for Provide {
    type FreeRole = Write;
    type UsedRole = Read;

    const ROLE: RingBuffersRoleValue = RingBuffersRoleValue::Provide;

    fn default_initialization_strategy() -> InitializationStrategy {
        InitializationStrategy::UseAndWriteState(Default::default())
    }
}

impl RingBuffersRole for Use {
    type FreeRole = Read;
    type UsedRole = Write;

    const ROLE: RingBuffersRoleValue = RingBuffersRoleValue::Use;

    fn default_initialization_strategy() -> InitializationStrategy {
        InitializationStrategy::ReadState
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum RingBuffersRoleValue {
    Provide,
    Use,
}

impl RingBuffersRoleValue {
    pub fn is_provide(self) -> bool {
        self == Self::Provide
    }

    pub fn is_use(self) -> bool {
        self == Self::Use
    }
}

pub enum Write {}
pub enum Read {}

pub trait RingBufferRole: RingBufferRoleSealed {
    const ROLE: RingBufferRoleValue;
}

impl RingBufferRole for Write {
    const ROLE: RingBufferRoleValue = RingBufferRoleValue::Write;
}

impl RingBufferRole for Read {
    const ROLE: RingBufferRoleValue = RingBufferRoleValue::Read;
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum RingBufferRoleValue {
    Write,
    Read,
}

impl RingBufferRoleValue {
    pub fn is_write(self) -> bool {
        self == Self::Write
    }

    pub fn is_read(self) -> bool {
        self == Self::Read
    }
}

use sealing::{RingBufferRoleSealed, RingBuffersRoleSealed};

mod sealing {
    use super::*;

    pub trait RingBuffersRoleSealed {}

    impl RingBuffersRoleSealed for Provide {}
    impl RingBuffersRoleSealed for Use {}

    pub trait RingBufferRoleSealed {}

    impl RingBufferRoleSealed for Write {}
    impl RingBufferRoleSealed for Read {}
}
