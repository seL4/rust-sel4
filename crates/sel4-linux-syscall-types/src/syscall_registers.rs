use core::ffi::VaList;

pub type SyscallRegisterWord = isize;

pub trait SyscallRegisters {
    fn next_register_word(&mut self) -> Option<SyscallRegisterWord>;

    fn next_register_value<T>(&mut self) -> Option<T>
    where
        T: SyscallRegisterValue,
    {
        self.next_register_word().map(T::from_word)
    }
}

pub trait SyscallRegisterValue {
    fn from_word(word: isize) -> Self;
}

macro_rules! impl_syscall_register_value {
    ($t:ty) => {
        impl SyscallRegisterValue for $t {
            fn from_word(word: isize) -> Self {
                word as Self
            }
        }
    };
}

impl_syscall_register_value!(isize);
impl_syscall_register_value!(usize);
impl_syscall_register_value!(i32);
impl_syscall_register_value!(u32);
impl_syscall_register_value!(i16);
impl_syscall_register_value!(u16);
impl_syscall_register_value!(i8);
impl_syscall_register_value!(u8);

#[cfg(target_pointer_width = "64")]
impl_syscall_register_value!(i64);

#[cfg(target_pointer_width = "64")]
impl_syscall_register_value!(u64);

impl<T> SyscallRegisterValue for *const T {
    fn from_word(word: isize) -> Self {
        word as Self
    }
}

impl<T> SyscallRegisterValue for *mut T {
    fn from_word(word: isize) -> Self {
        word as Self
    }
}

pub struct IteratorAsSyscallRegisters<T>(T);

impl<T> IteratorAsSyscallRegisters<T> {
    pub unsafe fn new(it: T) -> Self {
        Self(it)
    }
}

impl<T: Iterator<Item = SyscallRegisterWord>> SyscallRegisters for IteratorAsSyscallRegisters<T> {
    fn next_register_word(&mut self) -> Option<SyscallRegisterWord> {
        self.0.next()
    }
}

pub struct VaListAsSyscallRegisters<'a, 'f>(VaList<'a, 'f>);

impl<'a, 'f> VaListAsSyscallRegisters<'a, 'f> {
    pub unsafe fn new(va_list: VaList<'a, 'f>) -> Self {
        Self(va_list)
    }
}

impl<'a, 'f> SyscallRegisters for VaListAsSyscallRegisters<'a, 'f> {
    fn next_register_word(&mut self) -> Option<SyscallRegisterWord> {
        Some(unsafe { self.0.arg() })
    }
}
