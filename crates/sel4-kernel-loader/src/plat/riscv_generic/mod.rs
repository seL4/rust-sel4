use crate::plat::Plat;

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn put_char(c: u8) {
        sbi::legacy::console_putchar(c)
    }

    fn put_char_without_synchronization(c: u8) {
        sbi::legacy::console_putchar(c)
    }

    fn start_secondary_core(_core_id: usize, _sp: usize) {
        unimplemented!()
    }
}
