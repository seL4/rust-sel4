//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]

extern crate alloc;

mod compiler_builtins_supplement;
mod dummy_custom_getrandom;
mod get_current_time_impl;
mod no_server_cert_verifier;

pub use dummy_custom_getrandom::seed_dummy_custom_getrandom;
pub use get_current_time_impl::GetCurrentTimeImpl;
pub use no_server_cert_verifier::NoServerCertVerifier;
