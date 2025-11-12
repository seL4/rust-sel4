//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Rust project contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Config {
    pub run_ignored: RunIgnored,
}

/// Whether ignored test should be run or not
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RunIgnored {
    Yes,
    No,
    /// Run only ignored tests
    Only,
}

impl Default for RunIgnored {
    fn default() -> Self {
        Self::No
    }
}
