//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_if};

use crate::{sys, NoExplicitInvocationContext};

#[sel4_cfg(KERNEL_MCS)]
use crate::cap;

#[sel4_cfg(not(KERNEL_MCS))]
use crate::{InvocationContext, MessageInfo};

/// Configuration-dependant alias for conveying reply authority to syscalls.
pub type ReplyAuthority<C = NoExplicitInvocationContext> = ReplyAuthorityImpl<C>;

sel4_cfg_if! {
    if #[sel4_cfg(KERNEL_MCS)] {
        pub type ReplyAuthorityImpl<C> = cap::Reply<C>;

        impl<C> ReplyAuthority<C> {
            pub(crate) fn into_sys_reply_authority(self) -> sys::ReplyAuthority {
                self.bits()
            }
        }
    } else {
        pub type ReplyAuthorityImpl<C> = ImplicitReplyAuthority<C>;

        impl<C> ReplyAuthority<C> {
            pub(crate) fn into_sys_reply_authority(self) -> sys::ReplyAuthority {
            }
        }

        /// Under this configuration, no reply authority is required.
        #[derive(Default)]
        pub struct ImplicitReplyAuthority<C> {
            invocation_context: C,
        }

        impl<C> ImplicitReplyAuthority<C> {
            pub const fn new(invocation_context: C) -> Self {
                Self {
                    invocation_context,
                }
            }

            pub fn into_invocation_context(self) -> C {
                self.invocation_context
            }
        }

        impl<C: InvocationContext> ImplicitReplyAuthority<C> {
            /// Corresponds to `seL4_Reply`.
            pub fn reply(self, info: MessageInfo) {
                self.into_invocation_context()
                    .with_context(|ipc_buffer| ipc_buffer.inner_mut().seL4_Reply(info.into_inner()))
            }
        }

        impl ConveysReplyAuthority for () {
            type C = NoExplicitInvocationContext;

            fn into_reply_authority(self) -> ReplyAuthority<Self::C> {
                ImplicitReplyAuthority::default()
            }
        }
    }
}

/// Trait for types from which [`ReplyAuthority`] can be derived.
pub trait ConveysReplyAuthority {
    type C;

    fn into_reply_authority(self) -> ReplyAuthority<Self::C>;
}

impl<C> ConveysReplyAuthority for ReplyAuthority<C> {
    type C = C;

    fn into_reply_authority(self) -> ReplyAuthority<Self::C> {
        self
    }
}
