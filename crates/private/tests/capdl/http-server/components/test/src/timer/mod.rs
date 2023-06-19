use tests_capdl_http_server_components_test_sp804_driver::Driver;

use crate::Config;

pub fn init(config: &Config) -> Driver {
    unsafe {
        Driver::new(
            config.timer_mmio_vaddr as *mut (),
            config.timer_freq.try_into().unwrap(),
        )
    }
}
