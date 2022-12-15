use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicUsize, Ordering};

#[used]
#[no_mangle]
static mut spin_table_secondary_core_stack: usize = 0;

pub(crate) fn start_secondary_core(spin_table: &[usize], core_id: usize, sp: usize) {
    unsafe {
        spin_table_secondary_core_stack = sp;

        let start = (spin_table_secondary_core_entry as *const SecondaryCoreStartFn).to_bits();
        let start_ptr = <*mut usize>::from_bits(spin_table[core_id]);

        // Emits strl instruction. Ensures jump address is observed by spinning
        // core only after stack address, without the need for an explicit barrier.
        AtomicUsize::from_mut(&mut *start_ptr).store(start, Ordering::Release);

        dc_cvac(start_ptr.to_bits());
        dc_cvac((&spin_table_secondary_core_stack as *const usize).to_bits());

        // Barrier ensure both strl and dc cvac happen before sev
        asm!("dsb sy");
        asm!("sev");
    }
}

type SecondaryCoreStartFn = extern "C" fn() -> !;

extern "C" {
    fn spin_table_secondary_core_entry() -> !;
}

global_asm! {
    r#"
        .global spin_table_secondary_core_entry
        .extern spin_table_secondary_core_stack
        .extern secondary_entry

        .section .text

        spin_table_secondary_core_entry:
            ldr x9, =spin_table_secondary_core_stack
            ldr x9, [x9]
            mov sp, x9
            b secondary_entry
    "#
}

// helpers

unsafe fn dc_cvac(vaddr: usize) {
    asm!("dc cvac, {vaddr}", vaddr = in(reg) vaddr);
}
