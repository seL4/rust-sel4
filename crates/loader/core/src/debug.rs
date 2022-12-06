use crate::plat;

pub(crate) fn init() {
    plat::debug::init()
}

pub(crate) fn put_char(c: u8) {
    plat::debug::put_char(c)
}
