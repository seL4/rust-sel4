#![no_std]
#![feature(thread_local)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]
#![feature(strict_provenance)]
#![feature(array_methods)]
#![feature(variant_count)]

pub use sel4_config::{
    self as config, sel4_cfg, sel4_cfg_bool, sel4_cfg_if, sel4_cfg_str, sel4_cfg_usize,
};

pub use sel4_sys as sys;

mod arch;
mod bootinfo;
mod cap_rights;
mod cnode_cap_data;
mod cptr;
mod endpoint;
mod error;
mod helper_macros;
mod invocation_context;
mod invocations;
mod ipc_buffer;
mod message_info;
mod misc;
mod object;

pub mod fault;

pub use bootinfo::{
    BootInfo, BootInfoExtraStructure, BootInfoExtraStructureId, InitCSpaceSlot, UntypedDesc,
};
pub use cap_rights::{CapRights, CapRightsBuilder};
pub use cnode_cap_data::CNodeCapData;
pub use cptr::{
    cap_type, local_cptr, CPtr, CPtrBits, CPtrWithDepth, CapType, LocalCPtr, RelativeCPtr,
};
pub use endpoint::{reply, CallWithMRs, FastMessages, RecvWithMRs};
pub use error::{Error, Result};
pub use invocation_context::{InvocationContext, NoExplicitInvocationContext, NoInvocationContext};
pub use ipc_buffer::IPCBuffer;
pub use message_info::{MessageInfo, MessageInfoBuilder};
pub use misc::{r#yield, Badge, Word, WORD_SIZE};
pub use object::{ObjectBlueprint, ObjectType};

pub use arch::top_level::*;

#[doc(no_inline)]
pub use local_cptr::*;

#[doc(no_inline)]
pub use fault::*;

pub(crate) use helper_macros::{declare_cap_type, declare_local_cptr_alias, newtype_methods};

sel4_cfg_if! {
    if #[cfg(DEBUG_BUILD)] {
        mod debug;
        mod fmt;

        pub use debug::{debug_put_char, debug_snapshot};
    }
}

sel4_cfg_if! {
    if #[cfg(ENABLE_BENCHMARKS)] {
        mod benchmark;

        pub use benchmark::{
            benchmark_reset_log,
            benchmark_finalize_log,
            benchmark_set_log_buffer,
            benchmark_get_thread_utilisation,
            benchmark_reset_thread_utilisation,
            benchmark_dump_all_thread_utilisation,
            benchmark_reset_all_thread_utilisation,
        };
    }
}

#[cfg(feature = "state")]
mod state;

#[cfg(feature = "state")]
pub use state::{
    set_ipc_buffer, with_ipc_buffer, with_ipc_buffer_mut, ImplicitInvocationContext, IPC_BUFFER,
};

#[doc(hidden)]
pub mod _private {
    #[sel4_config::sel4_cfg(DEBUG_BUILD)]
    pub use super::fmt::_private as fmt;
}
