//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_if};

use crate::sys;

#[sel4_cfg(KERNEL_MCS)]
use crate::Reply;

sel4_cfg_if! {
    if #[sel4_cfg(KERNEL_MCS)] {
        pub type ReplyAuthority = Reply;

        impl ReplyAuthority {
            pub(crate) fn into_sys_reply_authority(self) -> sys::ReplyAuthority {
                self.bits()
            }
        }
    } else {
        pub type ReplyAuthority = ImplicitReplyAuthority;

        impl ReplyAuthority {
            pub(crate) fn into_sys_reply_authority(self) -> sys::ReplyAuthority {
            }
        }

        pub struct ImplicitReplyAuthority;

        impl ConveysReplyAuthority for () {
            fn into_reply_authority(self) -> ReplyAuthority {
                ImplicitReplyAuthority
            }
        }
    }
}

pub trait ConveysReplyAuthority {
    fn into_reply_authority(self) -> ReplyAuthority;
}

impl ConveysReplyAuthority for ReplyAuthority {
    fn into_reply_authority(self) -> ReplyAuthority {
        self
    }
}
