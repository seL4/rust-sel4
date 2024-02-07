//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;
use core::ffi::c_int;

use sel4_config::sel4_cfg;

use super::sys_id_to_word;
use crate::{seL4_MessageInfo, seL4_Word};

// NOTE
// asm!() does not allow r6 to be used for input or output operands, because it's sometimes used by LLVM.

pub fn sys_send(
    sys: c_int,
    dest: seL4_Word,
    info_arg: seL4_MessageInfo,
    mr0: seL4_Word,
    mr1: seL4_Word,
    mr2: seL4_Word,
    mr3: seL4_Word,
) {
    unsafe {
        asm!("swi 0",
            in("r7") sys_id_to_word(sys),
            in("r0") dest,
            in("r1") info_arg.into_word(),
            in("r2") mr0,
            in("r3") mr1,
            in("r4") mr2,
            in("r5") mr3,
        );
    }
}

#[sel4_cfg(not(KERNEL_MCS))]
pub fn sys_reply(
    sys: c_int,
    info_arg: seL4_MessageInfo,
    mr0: seL4_Word,
    mr1: seL4_Word,
    mr2: seL4_Word,
    mr3: seL4_Word,
) {
    unsafe {
        asm!("swi 0",
            in("r7") sys_id_to_word(sys),
            in("r1") info_arg.into_word(),
            in("r2") mr0,
            in("r3") mr1,
            in("r4") mr2,
            in("r5") mr3,
        );
    }
}

pub fn sys_send_null(sys: c_int, src: seL4_Word, info_arg: seL4_MessageInfo) {
    unsafe {
        asm!("swi 0",
            in("r7") sys_id_to_word(sys),
            in("r0") src,
            in("r1") info_arg.into_word(),
        );
    }
}

pub fn sys_recv(
    sys: c_int,
    src: seL4_Word,
    out_mr0: &mut seL4_Word,
    out_mr1: &mut seL4_Word,
    out_mr2: &mut seL4_Word,
    out_mr3: &mut seL4_Word,
    reply: seL4_Word,
) -> (seL4_MessageInfo, seL4_Word) {
    let out_info: seL4_Word;
    let out_badge: seL4_Word;
    unsafe {
        asm!(
            "mov r10, r6",
            "mov r6, r9",
            "swi 0",
            "mov r6, r10",
            in("r7") sys_id_to_word(sys),
            inout("r0") src => out_badge,
            out("r1") out_info,
            out("r2") *out_mr0,
            out("r3") *out_mr1,
            out("r4") *out_mr2,
            out("r5") *out_mr3,
            in("r9") reply,
            inout("r10") 0 => _,
        );
    }
    (seL4_MessageInfo::from_word(out_info), out_badge)
}

pub fn sys_send_recv(
    sys: c_int,
    dest: seL4_Word,
    info_arg: seL4_MessageInfo,
    in_out_mr0: &mut seL4_Word,
    in_out_mr1: &mut seL4_Word,
    in_out_mr2: &mut seL4_Word,
    in_out_mr3: &mut seL4_Word,
    reply: seL4_Word,
) -> (seL4_MessageInfo, seL4_Word) {
    let out_info: seL4_Word;
    let out_badge: seL4_Word;
    unsafe {
        asm!(
            "mov r10, r6",
            "mov r6, r9",
            "swi 0",
            "mov r6, r10",
            in("r7") sys_id_to_word(sys),
            inout("r0") dest => out_badge,
            inout("r1") info_arg.into_word() => out_info,
            inout("r2") *in_out_mr0,
            inout("r3") *in_out_mr1,
            inout("r4") *in_out_mr2,
            inout("r5") *in_out_mr3,
            in("r9") reply,
            inout("r10") 0 => _,
        );
    }
    (seL4_MessageInfo::from_word(out_info), out_badge)
}

#[sel4_cfg(KERNEL_MCS)]
pub fn sys_nb_send_recv(
    sys: c_int,
    dest: seL4_Word,
    src: seL4_Word,
    info_arg: seL4_MessageInfo,
    in_out_mr0: &mut seL4_Word,
    in_out_mr1: &mut seL4_Word,
    in_out_mr2: &mut seL4_Word,
    in_out_mr3: &mut seL4_Word,
    reply: seL4_Word,
) -> (seL4_MessageInfo, seL4_Word) {
    let out_info: seL4_Word;
    let out_badge: seL4_Word;
    unsafe {
        asm!(
            "mov r10, r6",
            "mov r6, r9",
            "swi 0",
            "mov r6, r10",
            in("r7") sys_id_to_word(sys),
            inout("r0") src => out_badge,
            inout("r1") info_arg.into_word() => out_info,
            inout("r2") *in_out_mr0,
            inout("r3") *in_out_mr1,
            inout("r4") *in_out_mr2,
            inout("r5") *in_out_mr3,
            in("r9") reply,
            in("r8") dest,
            inout("r10") 0 => _,
        );
    }
    (seL4_MessageInfo::from_word(out_info), out_badge)
}

pub fn sys_null(sys: c_int) {
    unsafe {
        asm!("swi 0",
            in("r7") sys_id_to_word(sys),
        );
    }
}
