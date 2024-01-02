//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use alloc::borrow::Cow;

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;

pub(crate) mod types;

use types::Config;

static CONFIG: ImmediateSyncOnceCell<Config> = ImmediateSyncOnceCell::new();

pub fn set_config(config: Config) {
    CONFIG.set(config).unwrap_or_else(|_| panic!())
}

pub(crate) fn get_config() -> Cow<'static, Config> {
    CONFIG
        .get()
        .map(Cow::Borrowed)
        .unwrap_or_else(|| Default::default())
}
