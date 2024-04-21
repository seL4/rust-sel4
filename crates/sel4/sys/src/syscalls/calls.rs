//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_int;
use core::sync::atomic::{compiler_fence, Ordering};

use sel4_config::{sel4_cfg, sel4_cfg_if};

use crate::{
    seL4_CPtr, seL4_IPCBuffer, seL4_MessageInfo, seL4_Uint32, seL4_Word, syscall_id,
    ReplyAuthority, WaitMessageInfo,
};

#[allow(unused_imports)]
use crate::seL4_Error;

use super::helpers::*;

const UNUSED_REPLY_ARG: seL4_Word = 0;

fn reply_authority_to_sys_arg(#[allow(unused_variables)] authority: ReplyAuthority) -> seL4_Word {
    sel4_cfg_if! {
        if #[cfg(KERNEL_MCS)] {
            authority
        } else {
            UNUSED_REPLY_ARG
        }
    }
}

macro_rules! fill_mrs_from_ipc_buffer {
    ($ipcbuf:ident, $mr0:ident, $mr1:ident, $mr2:ident, $mr3:ident) => {
        $mr0 = $ipcbuf.get_mr(0);
        $mr1 = $ipcbuf.get_mr(1);
        $mr2 = $ipcbuf.get_mr(2);
        $mr3 = $ipcbuf.get_mr(3);
    };
}

macro_rules! empty_mrs_to_ipc_buffer {
    ($ipcbuf:ident, $mr0:ident, $mr1:ident, $mr2:ident, $mr3:ident) => {
        $ipcbuf.set_mr(0, $mr0);
        $ipcbuf.set_mr(1, $mr1);
        $ipcbuf.set_mr(2, $mr2);
        $ipcbuf.set_mr(3, $mr3);
    };
}

#[rustfmt::skip]
macro_rules! fill_mrs_from_args {
    (
        $msg_info:ident,
        $mr0:ident, $mr1:ident, $mr2:ident, $mr3:ident,
        $msg0:ident, $msg1:ident, $msg2:ident, $msg3:ident,
    ) => {
        $mr0 = 0;
        $mr1 = 0;
        $mr2 = 0;
        $mr3 = 0;

        let n = $msg_info.get_length();

        match &$msg0 {
            Some(msg) if n > 0 => $mr0 = **msg,
            _ => (),
        }
        match &$msg1 {
            Some(msg) if n > 1 => $mr1 = **msg,
            _ => (),
        }
        match &$msg2 {
            Some(msg) if n > 2 => $mr2 = **msg,
            _ => (),
        }
        match &$msg3 {
            Some(msg) if n > 3 => $mr3 = **msg,
            _ => (),
        }
    };
}

macro_rules! fill_mrs_from_in_args {
    (
        $msg_info:ident,
        $mr0:ident, $mr1:ident, $mr2:ident, $mr3:ident,
        $msg0:ident, $msg1:ident, $msg2:ident, $msg3:ident,
    ) => {
        $mr0 = $msg_info.msg_helper($msg0, 0);
        $mr1 = $msg_info.msg_helper($msg1, 1);
        $mr2 = $msg_info.msg_helper($msg2, 2);
        $mr3 = $msg_info.msg_helper($msg3, 3);
    };
}

macro_rules! empty_mrs_to_args {
    (
        $mr0:ident, $mr1:ident, $mr2:ident, $mr3:ident,
        $msg0:ident, $msg1:ident, $msg2:ident, $msg3:ident,
    ) => {
        if let Some(msg) = $msg0 {
            *msg = $mr0;
        }
        if let Some(msg) = $msg1 {
            *msg = $mr1;
        }
        if let Some(msg) = $msg2 {
            *msg = $mr2;
        }
        if let Some(msg) = $msg3 {
            *msg = $mr3;
        }
    };
}

// HACK
macro_rules! fence {
    () => {
        compiler_fence(Ordering::SeqCst);
    };
}

fn sys_send_recv_simple(sys_id: c_int, arg: seL4_Word) -> seL4_Word {
    let mut mr0 = 0;
    let mut mr1 = 0;
    let mut mr2 = 0;
    let mut mr3 = 0;

    let (_msg_info, ret) = sys_send_recv(
        sys_id,
        arg,
        seL4_MessageInfo::new(0, 0, 0, 0),
        &mut mr0,
        &mut mr1,
        &mut mr2,
        &mut mr3,
        UNUSED_REPLY_ARG,
    );

    ret
}

impl seL4_IPCBuffer {
    pub fn seL4_Send(&mut self, dest: seL4_CPtr, msg_info: seL4_MessageInfo) {
        let mr0;
        let mr1;
        let mr2;
        let mr3;

        fill_mrs_from_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        sys_send(syscall_id::Send, dest, msg_info, mr0, mr1, mr2, mr3)
    }

    pub fn seL4_SendWithMRs(
        &mut self,
        dest: seL4_CPtr,
        msg_info: seL4_MessageInfo,
        msg0: Option<seL4_Word>,
        msg1: Option<seL4_Word>,
        msg2: Option<seL4_Word>,
        msg3: Option<seL4_Word>,
    ) {
        seL4_SendWithMRsWithoutIPCBuffer(dest, msg_info, msg0, msg1, msg2, msg3)
    }

    pub fn seL4_NBSend(&mut self, dest: seL4_CPtr, msg_info: seL4_MessageInfo) {
        let mr0;
        let mr1;
        let mr2;
        let mr3;

        fill_mrs_from_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        sys_send(syscall_id::NBSend, dest, msg_info, mr0, mr1, mr2, mr3)
    }

    pub fn seL4_NBSendWithMRs(
        &mut self,
        dest: seL4_CPtr,
        msg_info: seL4_MessageInfo,
        msg0: Option<seL4_Word>,
        msg1: Option<seL4_Word>,
        msg2: Option<seL4_Word>,
        msg3: Option<seL4_Word>,
    ) {
        seL4_NBSendWithMRsWithoutIPCBuffer(dest, msg_info, msg0, msg1, msg2, msg3)
    }

    #[sel4_cfg(not(KERNEL_MCS))]
    pub fn seL4_Reply(&mut self, msg_info: seL4_MessageInfo) {
        let mr0;
        let mr1;
        let mr2;
        let mr3;

        fill_mrs_from_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        sys_reply(syscall_id::Reply, msg_info, mr0, mr1, mr2, mr3)
    }

    #[sel4_cfg(not(KERNEL_MCS))]
    pub fn seL4_ReplyWithMRs(
        &mut self,
        msg_info: seL4_MessageInfo,
        msg0: Option<seL4_Word>,
        msg1: Option<seL4_Word>,
        msg2: Option<seL4_Word>,
        msg3: Option<seL4_Word>,
    ) {
        seL4_ReplyWithMRsWithoutIPCBuffer(msg_info, msg0, msg1, msg2, msg3)
    }

    pub fn seL4_Signal(&mut self, dest: seL4_CPtr) {
        let msg_info = seL4_MessageInfo::new(0, 0, 0, 0);

        sys_send_null(syscall_id::Send, dest, msg_info)
    }

    pub fn seL4_Recv(
        &mut self,
        src: seL4_CPtr,
        reply_authority: ReplyAuthority,
    ) -> (seL4_MessageInfo, seL4_Word) {
        let mut mr0 = 0;
        let mut mr1 = 0;
        let mut mr2 = 0;
        let mut mr3 = 0;

        let ret = sys_recv(
            syscall_id::Recv,
            src,
            &mut mr0,
            &mut mr1,
            &mut mr2,
            &mut mr3,
            reply_authority_to_sys_arg(reply_authority),
        );

        empty_mrs_to_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        ret
    }

    pub fn seL4_RecvWithMRs(
        &mut self,
        src: seL4_CPtr,
        msg0: Option<&mut seL4_Word>,
        msg1: Option<&mut seL4_Word>,
        msg2: Option<&mut seL4_Word>,
        msg3: Option<&mut seL4_Word>,
        reply_authority: ReplyAuthority,
    ) -> (seL4_MessageInfo, seL4_Word) {
        seL4_RecvWithMRsWithoutIPCBuffer(src, msg0, msg1, msg2, msg3, reply_authority)
    }

    pub fn seL4_NBRecv(
        &mut self,
        src: seL4_CPtr,
        reply_authority: ReplyAuthority,
    ) -> (seL4_MessageInfo, seL4_Word) {
        let mut mr0 = 0;
        let mut mr1 = 0;
        let mut mr2 = 0;
        let mut mr3 = 0;

        let ret = sys_recv(
            syscall_id::NBRecv,
            src,
            &mut mr0,
            &mut mr1,
            &mut mr2,
            &mut mr3,
            reply_authority_to_sys_arg(reply_authority),
        );

        empty_mrs_to_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        ret
    }

    pub fn seL4_Call(&mut self, dest: seL4_CPtr, msg_info: seL4_MessageInfo) -> seL4_MessageInfo {
        let mut mr0;
        let mut mr1;
        let mut mr2;
        let mut mr3;

        fill_mrs_from_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        let (out_msg_info, _badge) = sys_send_recv(
            syscall_id::Call,
            dest,
            msg_info,
            &mut mr0,
            &mut mr1,
            &mut mr2,
            &mut mr3,
            UNUSED_REPLY_ARG,
        );

        empty_mrs_to_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        out_msg_info
    }

    pub fn seL4_CallWithMRs(
        &mut self,
        dest: seL4_CPtr,
        msg_info: seL4_MessageInfo,
        msg0: Option<&mut seL4_Word>,
        msg1: Option<&mut seL4_Word>,
        msg2: Option<&mut seL4_Word>,
        msg3: Option<&mut seL4_Word>,
    ) -> seL4_MessageInfo {
        seL4_CallWithMRsWithoutIPCBuffer(dest, msg_info, msg0, msg1, msg2, msg3)
    }

    pub fn seL4_ReplyRecv(
        &mut self,
        src: seL4_CPtr,
        msg_info: seL4_MessageInfo,
        reply_authority: ReplyAuthority,
    ) -> (seL4_MessageInfo, seL4_Word) {
        let mut mr0;
        let mut mr1;
        let mut mr2;
        let mut mr3;

        fill_mrs_from_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        let ret = sys_send_recv(
            syscall_id::ReplyRecv,
            src,
            msg_info,
            &mut mr0,
            &mut mr1,
            &mut mr2,
            &mut mr3,
            reply_authority_to_sys_arg(reply_authority),
        );

        empty_mrs_to_ipc_buffer!(self, mr0, mr1, mr2, mr3);

        ret
    }

    sel4_cfg_if! {
        if #[cfg(KERNEL_MCS)] {
            pub fn seL4_NBSendRecv(
                &mut self,
                dest: seL4_CPtr,
                msg_info: seL4_MessageInfo,
                src: seL4_CPtr,
                reply_authority: ReplyAuthority,
            ) -> (seL4_MessageInfo, seL4_Word) {
                let mut mr0;
                let mut mr1;
                let mut mr2;
                let mut mr3;

                fill_mrs_from_ipc_buffer!(self, mr0, mr1, mr2, mr3);

                let ret = sys_nb_send_recv(
                    syscall_id::NBSendRecv,
                    dest,
                    src,
                    msg_info,
                    &mut mr0,
                    &mut mr1,
                    &mut mr2,
                    &mut mr3,
                    reply_authority_to_sys_arg(reply_authority),
                );

                empty_mrs_to_ipc_buffer!(self, mr0, mr1, mr2, mr3);

                ret
            }

            pub fn seL4_NBSendWait(
                &mut self,
                dest: seL4_CPtr,
                msg_info: seL4_MessageInfo,
                src: seL4_CPtr,
            ) -> (seL4_MessageInfo, seL4_Word) {
                let mut mr0;
                let mut mr1;
                let mut mr2;
                let mut mr3;

                fill_mrs_from_ipc_buffer!(self, mr0, mr1, mr2, mr3);

                let ret = sys_nb_send_recv(
                    syscall_id::NBSendWait,
                    0,
                    src,
                    msg_info,
                    &mut mr0,
                    &mut mr1,
                    &mut mr2,
                    &mut mr3,
                    dest,
                );

                empty_mrs_to_ipc_buffer!(self, mr0, mr1, mr2, mr3);

                ret
            }

            pub fn seL4_Wait(&mut self, src: seL4_CPtr) -> (WaitMessageInfo, seL4_Word) {
                let mut mr0 = 0;
                let mut mr1 = 0;
                let mut mr2 = 0;
                let mut mr3 = 0;

                let ret = sys_recv(
                    syscall_id::Wait,
                    src,
                    &mut mr0,
                    &mut mr1,
                    &mut mr2,
                    &mut mr3,
                    UNUSED_REPLY_ARG,
                );

                empty_mrs_to_ipc_buffer!(self, mr0, mr1, mr2, mr3);

                ret
            }

            #[sel4_cfg(KERNEL_MCS)]
            pub fn seL4_WaitWithMRs(
                &mut self,
                src: seL4_CPtr,
                msg0: Option<&mut seL4_Word>,
                msg1: Option<&mut seL4_Word>,
                msg2: Option<&mut seL4_Word>,
                msg3: Option<&mut seL4_Word>,
            ) -> (WaitMessageInfo, seL4_Word) {
                seL4_WaitWithMRsWithoutIPCBuffer(src, msg0, msg1, msg2, msg3)
            }

            pub fn seL4_NBWait(&mut self, src: seL4_CPtr) -> (WaitMessageInfo, seL4_Word) {
                let mut mr0 = 0;
                let mut mr1 = 0;
                let mut mr2 = 0;
                let mut mr3 = 0;

                let ret = sys_recv(
                    syscall_id::NBWait,
                    src,
                    &mut mr0,
                    &mut mr1,
                    &mut mr2,
                    &mut mr3,
                    UNUSED_REPLY_ARG,
                );

                empty_mrs_to_ipc_buffer!(self, mr0, mr1, mr2, mr3);

                ret
            }
        } else {

            pub fn seL4_Wait(&mut self, src: seL4_CPtr) -> (WaitMessageInfo, seL4_Word) {
                let (_msg_info, badge) = self.seL4_Recv(src, ());
                ((), badge)
            }

        }
    }

    pub fn seL4_Poll(&mut self, src: seL4_CPtr) -> (seL4_MessageInfo, seL4_Word) {
        sel4_cfg_if! {
            if #[cfg(KERNEL_MCS)] {
                self.seL4_NBWait(src)
            } else {
                self.seL4_NBRecv(src, ())
            }
        }
    }
}

pub fn seL4_SendWithMRsWithoutIPCBuffer(
    dest: seL4_CPtr,
    msg_info: seL4_MessageInfo,
    msg0: Option<seL4_Word>,
    msg1: Option<seL4_Word>,
    msg2: Option<seL4_Word>,
    msg3: Option<seL4_Word>,
) {
    let mr0;
    let mr1;
    let mr2;
    let mr3;

    fill_mrs_from_in_args!(msg_info, mr0, mr1, mr2, mr3, msg0, msg1, msg2, msg3,);

    sys_send(syscall_id::Send, dest, msg_info, mr0, mr1, mr2, mr3)
}

pub fn seL4_NBSendWithMRsWithoutIPCBuffer(
    dest: seL4_CPtr,
    msg_info: seL4_MessageInfo,
    msg0: Option<seL4_Word>,
    msg1: Option<seL4_Word>,
    msg2: Option<seL4_Word>,
    msg3: Option<seL4_Word>,
) {
    let mr0;
    let mr1;
    let mr2;
    let mr3;

    fill_mrs_from_in_args!(msg_info, mr0, mr1, mr2, mr3, msg0, msg1, msg2, msg3,);

    sys_send(syscall_id::NBSend, dest, msg_info, mr0, mr1, mr2, mr3)
}

#[sel4_cfg(not(KERNEL_MCS))]
pub fn seL4_ReplyWithMRsWithoutIPCBuffer(
    msg_info: seL4_MessageInfo,
    msg0: Option<seL4_Word>,
    msg1: Option<seL4_Word>,
    msg2: Option<seL4_Word>,
    msg3: Option<seL4_Word>,
) {
    let mr0;
    let mr1;
    let mr2;
    let mr3;

    fill_mrs_from_in_args!(msg_info, mr0, mr1, mr2, mr3, msg0, msg1, msg2, msg3,);

    sys_reply(syscall_id::Reply, msg_info, mr0, mr1, mr2, mr3)
}

pub fn seL4_RecvWithMRsWithoutIPCBuffer(
    src: seL4_CPtr,
    msg0: Option<&mut seL4_Word>,
    msg1: Option<&mut seL4_Word>,
    msg2: Option<&mut seL4_Word>,
    msg3: Option<&mut seL4_Word>,
    reply_authority: ReplyAuthority,
) -> (seL4_MessageInfo, seL4_Word) {
    let mut mr0 = 0;
    let mut mr1 = 0;
    let mut mr2 = 0;
    let mut mr3 = 0;

    let ret = sys_recv(
        syscall_id::Recv,
        src,
        &mut mr0,
        &mut mr1,
        &mut mr2,
        &mut mr3,
        reply_authority_to_sys_arg(reply_authority),
    );

    empty_mrs_to_args!(mr0, mr1, mr2, mr3, msg0, msg1, msg2, msg3,);

    ret
}

pub fn seL4_CallWithMRsWithoutIPCBuffer(
    dest: seL4_CPtr,
    msg_info: seL4_MessageInfo,
    msg0: Option<&mut seL4_Word>,
    msg1: Option<&mut seL4_Word>,
    msg2: Option<&mut seL4_Word>,
    msg3: Option<&mut seL4_Word>,
) -> seL4_MessageInfo {
    let mut mr0;
    let mut mr1;
    let mut mr2;
    let mut mr3;

    fill_mrs_from_args!(msg_info, mr0, mr1, mr2, mr3, msg0, msg1, msg2, msg3,);

    let (out_msg_info, _badge) = sys_send_recv(
        syscall_id::Call,
        dest,
        msg_info,
        &mut mr0,
        &mut mr1,
        &mut mr2,
        &mut mr3,
        UNUSED_REPLY_ARG,
    );

    empty_mrs_to_args!(mr0, mr1, mr2, mr3, msg0, msg1, msg2, msg3,);

    out_msg_info
}

#[sel4_cfg(KERNEL_MCS)]
pub fn seL4_WaitWithMRsWithoutIPCBuffer(
    src: seL4_CPtr,
    msg0: Option<&mut seL4_Word>,
    msg1: Option<&mut seL4_Word>,
    msg2: Option<&mut seL4_Word>,
    msg3: Option<&mut seL4_Word>,
) -> (WaitMessageInfo, seL4_Word) {
    let mut mr0 = 0;
    let mut mr1 = 0;
    let mut mr2 = 0;
    let mut mr3 = 0;

    let ret = sys_recv(
        syscall_id::Wait,
        src,
        &mut mr0,
        &mut mr1,
        &mut mr2,
        &mut mr3,
        UNUSED_REPLY_ARG,
    );

    empty_mrs_to_args!(mr0, mr1, mr2, mr3, msg0, msg1, msg2, msg3,);

    ret
}

pub fn seL4_Yield() {
    sys_null(syscall_id::Yield);
    fence!();
}

sel4_cfg_if! {
    if #[cfg(DEBUG_BUILD)] {
        pub fn seL4_DebugPutChar(c: u8) {
            sys_send_recv_simple(syscall_id::DebugPutChar, c as seL4_Word);
        }

        pub fn seL4_GetClock() -> seL4_Word {
            sys_send_recv_simple(syscall_id::GetClock, 0)
        }

        pub fn seL4_DebugHalt() {
            sys_null(syscall_id::DebugHalt);
            fence!();
        }

        pub fn seL4_DebugSnapshot() {
            sys_null(syscall_id::DebugSnapshot);
            fence!();
        }

        pub fn seL4_DebugCapIdentify(cap: seL4_CPtr) -> seL4_Uint32 {
            sys_send_recv_simple(syscall_id::DebugCapIdentify, cap) as seL4_Uint32
        }

        pub fn seL4_DebugNameThread(tcb: seL4_CPtr, name: &[u8], ipc_buffer: &mut seL4_IPCBuffer) {
            let mut mr0 = 0;
            let mut mr1 = 0;
            let mut mr2 = 0;
            let mut mr3 = 0;

            let ipc_buffer_msg_bytes = ipc_buffer.msg_bytes_mut();
            ipc_buffer_msg_bytes[0..name.len()].copy_from_slice(name);
            ipc_buffer_msg_bytes[name.len()] = 0;

            let _ = sys_send_recv(
                syscall_id::DebugNameThread,
                tcb,
                seL4_MessageInfo::new(0, 0, 0, 0),
                &mut mr0,
                &mut mr1,
                &mut mr2,
                &mut mr3,
                UNUSED_REPLY_ARG,
            );
        }
    }
}

sel4_cfg_if! {
    if #[cfg(ENABLE_BENCHMARKS)] {
        pub fn seL4_BenchmarkResetLog() -> seL4_Error::Type {
            sys_send_recv_simple(
                syscall_id::BenchmarkResetLog,
                0,
            ) as seL4_Error::Type
        }

        pub fn seL4_BenchmarkFinalizeLog() -> seL4_Word {
            sys_send_recv_simple(
                syscall_id::BenchmarkFinalizeLog,
                0,
            )
        }

        pub fn seL4_BenchmarkSetLogBuffer(frame_cap: seL4_CPtr) -> seL4_Error::Type {
            sys_send_recv_simple(
                syscall_id::BenchmarkSetLogBuffer,
                frame_cap,
            ) as seL4_Error::Type
        }

        sel4_cfg_if! {
            if #[cfg(BENCHMARK_TRACK_UTILISATION)] {
                pub fn seL4_BenchmarkGetThreadUtilisation(tcb: seL4_CPtr) {
                    sys_send_recv_simple(
                        syscall_id::BenchmarkGetThreadUtilisation,
                        tcb,
                    );
                }

                pub fn seL4_BenchmarkResetThreadUtilisation(tcb: seL4_CPtr) {
                    sys_send_recv_simple(
                        syscall_id::BenchmarkResetThreadUtilisation,
                        tcb,
                    );
                }

                sel4_cfg_if! {
                    if #[cfg(DEBUG_BUILD)] {
                        pub fn seL4_BenchmarkDumpAllThreadsUtilisation() {
                            sys_send_recv_simple(
                                syscall_id::BenchmarkDumpAllThreadsUtilisation,
                                0,
                            );
                        }

                        pub fn seL4_BenchmarkResetAllThreadsUtilisation() {
                            sys_send_recv_simple(
                                syscall_id::BenchmarkResetAllThreadsUtilisation,
                                0,
                            );
                        }
                    }
                }
            }
        }
    }
}

sel4_cfg_if! {
    if #[cfg(SET_TLS_BASE_SELF)] {
        pub fn seL4_SetTLSBase(tls_base: seL4_Word) {
            let msg_info = seL4_MessageInfo::new(0, 0, 0, 0);
            sys_send_null(syscall_id::SetTLSBase, tls_base, msg_info)
        }
    }
}

sel4_cfg_if! {
    if #[cfg(UINTR)] {
        pub fn seL4_WakeSyscallHandler() {
            sys_send_recv_simple(syscall_id::WakeSyscallHandler, 0);
        }
    }
}




