pub(crate) fn init_platform_state_per_core(_core_id: usize) {}

pub(crate) fn start_secondary_core(_core_id: usize, _sp: usize) {
    unimplemented!()
}

pub(crate) mod debug {
    pub(crate) fn init() {}

    pub(crate) fn put_char(c: u8) {
        sbi::legacy::console_putchar(c)
    }

    pub(crate) fn put_char_without_synchronization(c: u8) {
        sbi::legacy::console_putchar(c)
    }
}
