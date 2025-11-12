//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::error::Error;
use core::fmt;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct PeerMisbehaviorError(());

impl PeerMisbehaviorError {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for PeerMisbehaviorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "peer misbehavior")
    }
}

impl Error for PeerMisbehaviorError {}
