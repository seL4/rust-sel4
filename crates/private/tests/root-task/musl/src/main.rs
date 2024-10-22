//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_main]
#![feature(thread_local)]

use core::mem::MaybeUninit;
use core::ops::Range;
use core::ptr;

use lock_api::Mutex;

use dlmalloc::Dlmalloc;
use sel4_dlmalloc::{StaticDlmallocAllocator, StaticHeap};
use sel4_linux_syscall_types::{ENOMEM, ENOSYS, MAP_ANONYMOUS, SEEK_CUR};
use sel4_musl::{
    set_syscall_handler, ParseSyscallError, Syscall, SyscallReturnValue, VaListAsSyscallArgs,
};
use sel4_root_task_with_std::{debug_print, debug_println, declare_root_task};
use sel4_sync_trivial::PanickingRawMutex;

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
            (if addr.is_null() {
                BRK_HEAP.start()
            } else if (BRK_HEAP.start()..BRK_HEAP.end()).contains(&addr.cast()) {
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
                (unsafe { MMAP_DLMALLOC.lock().malloc(len, 4096) }) as SyscallReturnValue
            } else {
                -ENOMEM
            }
        }
        Lseek { fd, offset, whence } => {
            assert!(whence == SEEK_CUR);
            assert!(offset == 0);
            assert!(0 <= fd && fd <= 2);
            0
        }
        Write { fd, buf, count } => {
            assert!(fd == 1 || fd == 2);
            for i in 0..(count as isize) {
                let c: u8 = unsafe { *buf.offset(i) };
                debug_print!("{}", c as char);
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

static MMAP_DLMALLOC: Mutex<
    PanickingRawMutex,
    Dlmalloc<StaticDlmallocAllocator<&'static StaticHeap<MMAP_HEAP_SIZE>>>,
> = Mutex::new(Dlmalloc::new_with_allocator(StaticDlmallocAllocator::new(
    &MMAP_HEAP,
)));
