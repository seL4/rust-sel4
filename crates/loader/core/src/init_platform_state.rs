use core::arch::asm;

use aarch64_cpu::registers::CurrentEL;
use tock_registers::interfaces::Readable;

extern "C" {
    fn el2_mmu_enable();
}

pub fn init_platform_state_per_core(core_id: usize) {
    let current_el = get_current_el();
    assert!(current_el == Some(CurrentEL::EL::Value::EL2));

    unsafe {
        set_tpidr(core_id);
    }
}

pub fn init_platform_state_per_core_after_which_no_syncronization(_core_id: usize) {
    unsafe {
        el2_mmu_enable();
    }
}

fn get_current_el() -> Option<CurrentEL::EL::Value> {
    CurrentEL.read_as_enum(CurrentEL::EL)
}

#[inline(never)] // never inline to work around issues with optimizer
unsafe fn set_tpidr(tpidr: usize) {
    asm!("msr tpidr_el1, {tpidr}", tpidr = in(reg) tpidr);
}
