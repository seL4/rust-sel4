#![no_std]
#![no_main]

use sel4cp::{main, NullHandler};

mod device;

use device::Pl011Device;

#[used]
#[no_mangle]
#[link_section = ".data"]
static mut pl011_mmio_base: usize = 0;

#[main]
fn main() -> NullHandler {
    sel4::debug_println!("pl011 driver");
    let device = Pl011Device::new(unsafe { pl011_mmio_base });
    device.put_char(b'x');
    device.put_char(b'y');
    device.put_char(b'z');
    device.put_char(b'\n');
    NullHandler::new()
}
