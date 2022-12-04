use core::arch::asm;
use core::ffi::c_int;

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
        asm!("svc 0",
            in("x7") sys_id_to_word(sys),
            in("x0") dest,
            in("x1") info_arg.into_word(),
            in("x2") mr0,
            in("x3") mr1,
            in("x4") mr2,
            in("x5") mr3,
        );
    }
}

pub fn sys_reply(
    sys: c_int,
    info_arg: seL4_MessageInfo,
    mr0: seL4_Word,
    mr1: seL4_Word,
    mr2: seL4_Word,
    mr3: seL4_Word,
) {
    unsafe {
        asm!("svc 0",
            in("x7") sys_id_to_word(sys),
            in("x1") info_arg.into_word(),
            in("x2") mr0,
            in("x3") mr1,
            in("x4") mr2,
            in("x5") mr3,
        );
    }
}

pub fn sys_send_null(
    sys: c_int,
    src: seL4_Word,
    info_arg: seL4_MessageInfo,
) {
    unsafe {
        asm!("svc 0",
            in("x7") sys_id_to_word(sys),
            in("x0") src,
            in("x1") info_arg.into_word(),
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
    _reply: seL4_Word,
) -> (seL4_MessageInfo, seL4_Word) {
    let out_info: seL4_Word;
    let out_badge: seL4_Word;
    unsafe {
        asm!("svc 0",
            in("x7") sys_id_to_word(sys),
            inout("x0") src => out_badge,
            out("x1") out_info,
            out("x2") *out_mr0,
            out("x3") *out_mr1,
            out("x4") *out_mr2,
            out("x5") *out_mr3,
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
    _reply: seL4_Word,
) -> (seL4_MessageInfo, seL4_Word) {
    let out_info: seL4_Word;
    let out_badge: seL4_Word;
    unsafe {
        asm!("svc 0",
            in("x7") sys_id_to_word(sys),
            inout("x0") dest => out_badge,
            inout("x1") info_arg.into_word() => out_info,
            inout("x2") *in_out_mr0,
            inout("x3") *in_out_mr1,
            inout("x4") *in_out_mr2,
            inout("x5") *in_out_mr3,
        );
    }
    (seL4_MessageInfo::from_word(out_info), out_badge)
}

pub fn sys_null(
    sys: c_int,
) {
    unsafe {
        asm!("svc 0",
            in("x7") sys_id_to_word(sys),
        );
    }
}
