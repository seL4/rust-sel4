//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;

use serde::Deserialize;

// TODO factor out into crates

type Ranges = Vec<Range<u64>>;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PlatformInfoForBuildSystem {
    pub(crate) memory: Ranges,
    #[allow(dead_code)]
    pub(crate) devices: Ranges,
}
