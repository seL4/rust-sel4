//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub use embedded_fat as fat;

#[derive(Default)]
pub struct DummyTimeSource(());

impl DummyTimeSource {
    pub fn new() -> Self {
        Self(())
    }
}

impl fat::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> fat::Timestamp {
        unimplemented!()
    }
}
