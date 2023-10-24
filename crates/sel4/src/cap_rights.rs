//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{newtype_methods, sys};

/// Corresponds to `seL4_CapRights_t`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapRights(sys::seL4_CapRights);

impl CapRights {
    newtype_methods!(sys::seL4_CapRights);

    pub fn new(grant_reply: bool, grant: bool, read: bool, write: bool) -> Self {
        Self::from_inner(sys::seL4_CapRights::new(
            grant_reply.into(),
            grant.into(),
            read.into(),
            write.into(),
        ))
    }

    pub fn none() -> Self {
        CapRightsBuilder::none().build()
    }

    pub fn all() -> Self {
        CapRightsBuilder::all().build()
    }

    pub fn read_write() -> Self {
        CapRightsBuilder::none().read(true).write(true).build()
    }

    pub fn read_only() -> Self {
        CapRightsBuilder::none().read(true).build()
    }

    pub fn write_only() -> Self {
        CapRightsBuilder::none().write(true).build()
    }
}

impl From<CapRightsBuilder> for CapRights {
    fn from(builder: CapRightsBuilder) -> Self {
        builder.build()
    }
}

/// Helper for constructing [`CapRights`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapRightsBuilder {
    grant_reply: bool,
    grant: bool,
    read: bool,
    write: bool,
}

impl CapRightsBuilder {
    pub fn none() -> Self {
        Default::default()
    }

    pub fn all() -> Self {
        Self {
            grant_reply: true,
            grant: true,
            read: true,
            write: true,
        }
    }

    pub fn build(self) -> CapRights {
        CapRights::new(self.grant_reply, self.grant, self.read, self.write)
    }

    pub fn grant_reply(mut self, can: bool) -> Self {
        self.grant_reply = can;
        self
    }

    pub fn grant(mut self, can: bool) -> Self {
        self.grant = can;
        self
    }

    pub fn read(mut self, can: bool) -> Self {
        self.read = can;
        self
    }

    pub fn write(mut self, can: bool) -> Self {
        self.write = can;
        self
    }
}
