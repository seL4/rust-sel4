//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

#![no_std]
#![feature(lang_items)]
#![feature(never_type)]
#![allow(internal_features)]

extern crate alloc;

mod config;
mod entry;
mod run_tests;
mod short_backtrace;

pub mod for_generated_code;

pub use {
    config::{set_config, types::*},
    entry::run_test_main,
};
