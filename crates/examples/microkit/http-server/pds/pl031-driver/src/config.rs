//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(dead_code)]

pub mod channels {
    use sel4_microkit::Channel;

    pub const DEVICE: Channel = Channel::new(0);
    pub const CLIENT: Channel = Channel::new(1);
}
