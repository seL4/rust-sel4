//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

mod cargo_manifest_policy;
mod easy_policy;
mod format;

pub use cargo_manifest_policy::cargo_manifest_policy;
pub use easy_policy::{EasyPolicy, TableRule, TableRuleOrdering};
pub use format::{Formatter, Policy};
