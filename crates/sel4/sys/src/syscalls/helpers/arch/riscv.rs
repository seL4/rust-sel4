//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;
use core::ffi::c_int;

use sel4_config::sel4_cfg;

use crate::{seL4_Word, seL4_MessageInfo};
use super::sys_id_to_word;

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
        asm!("ecall",
            in("a7") sys_id_to_word(sys),
            in("a0") dest,
            in("a1") info_arg.into_word(),
            in("a2") mr0,
            in("a3") mr1,
            in("a4") mr2,
            in("a5") mr3,
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
        asm!("ecall",
            in("a7") sys_id_to_word(sys),
            in("a1") info_arg.into_word(),
            in("a2") mr0,
            in("a3") mr1,
            in("a4") mr2,
            in("a5") mr3,
        );
    }
}

pub fn sys_send_null(
    sys: c_int,
    src: seL4_Word,
    info_arg: seL4_MessageInfo,
) {
    unsafe {
        asm!("ecall",
            in("a7") sys_id_to_word(sys),
            in("a0") src,
            in("a1") info_arg.into_word(),
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
        asm!("ecall",
            in("a7") sys_id_to_word(sys),
            inout("a0") src => out_badge,
            out("a1") out_info,
            out("a2") *out_mr0,
            out("a3") *out_mr1,
            out("a4") *out_mr2,
            out("a5") *out_mr3,
            in("a6") reply,
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
        asm!("ecall",
            in("a7") sys_id_to_word(sys),
            inout("a0") dest => out_badge,
            inout("a1") info_arg.into_word() => out_info,
            inout("a2") *in_out_mr0,
            inout("a3") *in_out_mr1,
            inout("a4") *in_out_mr2,
            inout("a5") *in_out_mr3,
            in("a6") reply,
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
        asm!("ecall",
            in("a7") sys_id_to_word(sys),
            inout("a0") src => out_badge,
            inout("a1") info_arg.into_word() => out_info,
            inout("a2") *in_out_mr0,
            inout("a3") *in_out_mr1,
            inout("a4") *in_out_mr2,
            inout("a5") *in_out_mr3,
            in("a6") reply,
            in("t0") dest,
        );
    }
    (seL4_MessageInfo::from_word(out_info), out_badge)
}

pub fn sys_null(
    sys: c_int,
) {
    unsafe {
        asm!("ecall",
            in("a7") sys_id_to_word(sys),
        );
    }
}
