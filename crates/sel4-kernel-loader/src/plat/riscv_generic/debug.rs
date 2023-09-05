pub(crate) fn init() {}

pub(crate) fn put_char(c: u8) {
    sbi::legacy::console_putchar(c)
}

pub(crate) fn put_char_without_synchronization(c: u8) {
    sbi::legacy::console_putchar(c)
}
