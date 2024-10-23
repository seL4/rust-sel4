//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::VaList;

pub type SyscallWordArg = usize;

pub trait SyscallArg {
    fn from_word(word: SyscallWordArg) -> Self;
}

macro_rules! impl_syscall_arg {
    ($t:ty) => {
        impl SyscallArg for $t {
            fn from_word(word: SyscallWordArg) -> Self {
                word as Self
            }
        }
    };
}

impl_syscall_arg!(isize);
impl_syscall_arg!(usize);
impl_syscall_arg!(i32);
impl_syscall_arg!(u32);
impl_syscall_arg!(i16);
impl_syscall_arg!(u16);
impl_syscall_arg!(i8);
impl_syscall_arg!(u8);

#[cfg(target_pointer_width = "64")]
impl_syscall_arg!(i64);

#[cfg(target_pointer_width = "64")]
impl_syscall_arg!(u64);

impl<T> SyscallArg for *const T {
    fn from_word(word: SyscallWordArg) -> Self {
        word as Self
    }
}

impl<T> SyscallArg for *mut T {
    fn from_word(word: SyscallWordArg) -> Self {
        word as Self
    }
}

pub trait SyscallArgs {
    fn next_word_arg(&mut self) -> Option<SyscallWordArg>;

    fn next_arg<T>(&mut self) -> Option<T>
    where
        T: SyscallArg,
    {
        self.next_word_arg().map(T::from_word)
    }
}

impl<T: SyscallArgs + ?Sized> SyscallArgs for &mut T {
    fn next_word_arg(&mut self) -> Option<SyscallWordArg> {
        T::next_word_arg(self)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IteratorAsSyscallArgs<T>(T);

impl<T> IteratorAsSyscallArgs<T> {
    pub fn new(it: T) -> Self {
        Self(it)
    }
}

impl<T: Iterator<Item = SyscallWordArg>> SyscallArgs for IteratorAsSyscallArgs<T> {
    fn next_word_arg(&mut self) -> Option<SyscallWordArg> {
        self.0.next()
    }
}

#[derive(Debug)]
pub struct VaListAsSyscallArgs<'a, 'f>(VaList<'a, 'f>);

impl<'a, 'f> VaListAsSyscallArgs<'a, 'f> {
    pub unsafe fn new(va_list: VaList<'a, 'f>) -> Self {
        Self(va_list)
    }
}

impl<'a, 'f> SyscallArgs for VaListAsSyscallArgs<'a, 'f> {
    fn next_word_arg(&mut self) -> Option<SyscallWordArg> {
        Some(unsafe { self.0.arg() })
    }
}
