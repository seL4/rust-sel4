//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]

extern crate alloc;

mod dummy_custom_getrandom;
mod no_server_cert_verifier;
mod time_provider_impl;

pub use dummy_custom_getrandom::seed_dummy_custom_getrandom;
pub use no_server_cert_verifier::NoServerCertVerifier;
pub use time_provider_impl::TimeProviderImpl;
