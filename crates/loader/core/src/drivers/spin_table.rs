use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicUsize, Ordering};

#[used]
#[no_mangle]
static mut spin_table_secondary_stack_bottom: usize = 0;

pub(crate) fn start_secondary_core(spin_table: &[usize], core_id: usize, sp: usize) {
    unsafe {
        spin_table_secondary_stack_bottom = sp;

        let start = (spin_table_secondary_entry as *const SpinTableSecondaryEntryFn).to_bits();
        let start_ptr = <*mut usize>::from_bits(spin_table[core_id]);

        // Emits strl instruction. Ensures jump address is observed by spinning
        // core only after stack address, without the need for an explicit barrier.
        AtomicUsize::from_mut(&mut *start_ptr).store(start, Ordering::Release);

        dc_cvac(start_ptr.to_bits());
        dc_cvac((&spin_table_secondary_stack_bottom as *const usize).to_bits());

        // Barrier ensure both strl and dc cvac happen before sev
        asm!("dsb sy");
        asm!("sev");
    }
}

type SpinTableSecondaryEntryFn = extern "C" fn() -> !;

extern "C" {
    fn spin_table_secondary_entry() -> !;
}

global_asm! {
    r#"
        .extern spin_table_secondary_stack_bottom
        .extern secondary_entry

        .section .text

        .global spin_table_secondary_entry
        spin_table_secondary_entry:
            ldr x9, =spin_table_secondary_stack_bottom
            ldr x9, [x9]
            mov sp, x9
            b secondary_entry
    "#
}

// helpers

unsafe fn dc_cvac(vaddr: usize) {
    asm!("dc cvac, {vaddr}", vaddr = in(reg) vaddr);
}
