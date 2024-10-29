//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

use zerocopy::{FromBytes, Immutable, IntoBytes};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, IntoBytes, FromBytes, Immutable)]
pub struct Descriptor {
    encoded_addr: usize,
    len: u32,
    _padding: [u8; 4],
    cookie: usize,
}

impl Descriptor {
    pub fn new(encoded_addr: usize, len: u32, cookie: usize) -> Self {
        Self {
            encoded_addr,
            len,
            _padding: [0; 4],
            cookie,
        }
    }

    pub fn from_encoded_addr_range(encoded_addr_range: Range<usize>, cookie: usize) -> Self {
        let encoded_addr = encoded_addr_range.start;
        let len = encoded_addr_range.len().try_into().unwrap();
        Self::new(encoded_addr, len, cookie)
    }

    pub fn encoded_addr(&self) -> usize {
        self.encoded_addr
    }

    pub fn set_encoded_addr(&mut self, encoded_addr: usize) {
        self.encoded_addr = encoded_addr;
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn set_len(&mut self, len: u32) {
        self.len = len;
    }

    pub fn cookie(&self) -> usize {
        self.cookie
    }

    pub fn set_cookie(&mut self, cookie: usize) {
        self.cookie = cookie;
    }

    pub fn encoded_addr_range(&self) -> Range<usize> {
        let start = self.encoded_addr();
        let len = self.len().try_into().unwrap();
        start..start.checked_add(len).unwrap()
    }
}
