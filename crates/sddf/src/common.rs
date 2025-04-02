//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct PeerMisbehaviorError(());

impl PeerMisbehaviorError {
    pub(crate) fn new() -> Self {
        Self(())
    }
}
