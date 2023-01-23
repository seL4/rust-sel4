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
        asm!(
            "mov r14, rsp",
            "syscall",
            "mov rsp, r14",
            in("rdx") sys_id_to_word(sys),
            in("rdi") dest,
            in("rsi") info_arg.into_word(),
            in("r10") mr0,
            in("r8") mr1,
            in("r9") mr2,
            in("r15") mr3,
            lateout("rcx") _,
            lateout("r11") _,
            lateout("r14") _,
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
        asm!(
            "mov r14, rsp",
            "syscall",
            "mov rsp, r14",
            in("rdx") sys_id_to_word(sys),
            in("rsi") info_arg.into_word(),
            in("r10") mr0,
            in("r8") mr1,
            in("r9") mr2,
            in("r15") mr3,
            lateout("rcx") _,
            lateout("r11") _,
            lateout("r14") _,
        );
    }
}

pub fn sys_send_null(
    sys: c_int,
    src: seL4_Word,
    info_arg: seL4_MessageInfo,
) {
    unsafe {
        asm!(
            "mov r14, rsp",
            "syscall",
            "mov rsp, r14",
            in("rdx") sys_id_to_word(sys),
            in("rdi") src,
            in("rsi") info_arg.into_word(),
            lateout("rcx") _,
            lateout("r11") _,
            lateout("r14") _,
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
            "mov r14, rsp",
            "syscall",
            "mov rsp, r14",
            in("rdx") sys_id_to_word(sys),
            inout("rdi") src => out_badge,
            out("rsi") out_info,
            out("r10") *out_mr0,
            out("r8") *out_mr1,
            out("r9") *out_mr2,
            out("r15") *out_mr3,
            in("r12") reply,
            lateout("rcx") _,
            lateout("r11") _,
            lateout("r14") _,
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
            "mov r14, rsp",
            "syscall",
            "mov rsp, r14",
            in("rdx") sys_id_to_word(sys),
            inout("rdi") dest => out_badge,
            inout("rsi") info_arg.into_word() => out_info,
            inout("r10") *in_out_mr0,
            inout("r8") *in_out_mr1,
            inout("r9") *in_out_mr2,
            inout("r15") *in_out_mr3,
            in("r12") reply,
            lateout("rcx") _,
            lateout("r11") _,
            lateout("r14") _,
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
            "mov r14, rsp",
            "syscall",
            "mov rsp, r14",
            in("rdx") sys_id_to_word(sys),
            inout("rdi") src => out_badge,
            inout("rsi") info_arg.into_word() => out_info,
            inout("r10") *in_out_mr0,
            inout("r8") *in_out_mr1,
            inout("r9") *in_out_mr2,
            inout("r15") *in_out_mr3,
            in("r12") reply,
            in("r13") dest,
            lateout("rcx") _,
            lateout("r11") _,
            lateout("r14") _,
        );
    }
    (seL4_MessageInfo::from_word(out_info), out_badge)
}

pub fn sys_null(
    sys: c_int,
) {
    unsafe {
        asm!(
            "mov r14, rsp",
            "syscall",
            "mov rsp, r14",
            in("rdx") sys_id_to_word(sys),
            lateout("rcx") _,
            lateout("r11") _,
            lateout("r14") _,
            options(nomem, nostack),
        );
    }
}
