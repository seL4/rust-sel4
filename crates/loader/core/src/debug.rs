use crate::plat;

pub(crate) fn init() {
    plat::debug::init()
}

pub(crate) fn put_char(c: u8) {
    plat::debug::put_char(c)
}

pub(crate) fn put_char_without_synchronization(c: u8) {
    plat::debug::put_char_without_synchronization(c)
}
