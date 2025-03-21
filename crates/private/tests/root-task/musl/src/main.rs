//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_main]
#![feature(thread_local)]
#![allow(unreachable_patterns)]
#![allow(unused_variables)]

use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::ffi::c_char;
use core::ptr;

use one_shot_mutex::sync::RawOneShotMutex;

use sel4_dlmalloc::{StaticDlmalloc, StaticHeap};
use sel4_linux_syscall_types::{ENOMEM, ENOSYS, MAP_ANONYMOUS, SEEK_CUR};
use sel4_musl::{
    set_syscall_handler, ParseSyscallError, Syscall, SyscallReturnValue, VaListAsSyscallArgs,
};
use sel4_root_task_with_std::{debug_print, debug_println, declare_root_task};

declare_root_task!(main = main);

fn main(_: &sel4::BootInfoPtr) -> ! {
    unsafe {
        set_syscall_handler(handle_syscall);
    }

    let x = vec![1, 2, 3];
    println!("x: {:?}", x);

    debug_println!("TEST_PASS");
    sel4::init_thread::suspend_self()
}

fn handle_syscall(
    syscall: Result<Syscall, ParseSyscallError<VaListAsSyscallArgs>>,
) -> SyscallReturnValue {
    match syscall {
        Ok(syscall) => {
            // debug_println!("{syscall:?}");
            handle_known_syscall(syscall)
        }
        Err(err) => {
            debug_println!("{err:?}");
            -ENOSYS
        }
    }
}

fn handle_known_syscall(syscall: Syscall) -> SyscallReturnValue {
    use Syscall::*;

    match syscall {
        Getuid | Geteuid | Getgid | Getegid => -ENOSYS,
        Brk { addr } => {
            let bounds = BRK_HEAP.bounds();
            (if addr.is_null() {
                bounds.start()
            } else if (bounds.start()..bounds.end()).contains(&addr.cast()) {
                addr.cast()
            } else {
                ptr::null()
            }) as SyscallReturnValue
        }
        Mmap {
            addr,
            len,
            prot,
            flag,
            fd,
            offset,
        } => {
            if flag & MAP_ANONYMOUS != 0 {
                (unsafe { MMAP_DLMALLOC.alloc(Layout::from_size_align(len, 4096).unwrap()) })
                    as SyscallReturnValue
            } else {
                -ENOMEM
            }
        }
        Lseek { fd, offset, whence } => {
            assert!(whence == SEEK_CUR);
            assert!(offset == 0);
            assert!((0..=2).contains(&fd));
            0
        }
        Write { fd, buf, count } => {
            assert!(fd == 1 || fd == 2);
            for i in 0..(count as isize) {
                let c: c_char = unsafe { *buf.offset(i) };
                debug_print!("{}", c as u8 as char);
            }
            count as SyscallReturnValue
        }
        Writev { fd, iov, iovcnt } => {
            assert!(fd == 1 || fd == 2);
            let mut ret: isize = 0;
            for i in 0..(iovcnt as isize) {
                let iov = unsafe { &*iov.offset(i) };
                for j in 0..(iov.iov_len as isize) {
                    let c: u8 = unsafe { *(iov.iov_base as *const u8).offset(j) };
                    debug_print!("{}", c as char);
                    ret += 1;
                }
            }
            ret as SyscallReturnValue
        }
        _ => panic!(),
    }
}

static BRK_HEAP: StaticHeap<{ 2 * 1024 * 1024 }> = StaticHeap::new();

const MMAP_HEAP_SIZE: usize = 2 * 1024 * 1024;

static MMAP_HEAP: StaticHeap<MMAP_HEAP_SIZE> = StaticHeap::new();

static MMAP_DLMALLOC: StaticDlmalloc<RawOneShotMutex> = StaticDlmalloc::new(MMAP_HEAP.bounds());
