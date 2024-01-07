//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::bf::SeL4Bitfield;

use crate::{seL4_MessageInfo, seL4_Word};

mod arch;

pub use arch::*;

impl seL4_MessageInfo {
    pub(crate) fn from_word(word: seL4_Word) -> Self {
        Self(SeL4Bitfield::new([word]))
    }

    pub(crate) fn into_word(self) -> seL4_Word {
        self.0.into_inner()[0]
    }

    pub(crate) fn msg_helper(&self, msg: Option<seL4_Word>, i: seL4_Word) -> seL4_Word {
        match msg {
            Some(msg) if i < self.get_length() => msg,
            _ => 0,
        }
    }
}
