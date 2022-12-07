use core::arch::global_asm;

use aligned::{Aligned, A16};

const PRIMARY_STACK_SIZE: usize = 4096 * 3;

#[no_mangle]
static mut __primary_stack: Aligned<A16, [u8; PRIMARY_STACK_SIZE]> =
    Aligned([0; PRIMARY_STACK_SIZE]);

#[no_mangle]
static __primary_stack_size: usize = PRIMARY_STACK_SIZE;

global_asm! {
    r#"
        .global _start;

        .extern main
        .extern __primary_stack
        .extern __primary_stack_size

        .section ".text.startup"

        _start:
            mrs     x0, mpidr_el1
            and     x0, x0, #0xf        // Check processor id
            cbz     x0, primary         // Hang for all non-primary CPU

        secondary_hang:
            wfe
            b       secondary_hang

        primary:
            // TODO GNU LD has __bss_start__ and __bss_end__ which feel more robust
            ldr	    x0, =__bss_start
            ldr	    x1, =_end

        primary_clear_bss_loop:
            str	    xzr, [x0], #8
            cmp	    x0, x1
            b.lt	primary_clear_bss_loop

            ldr     x9, =__primary_stack
            ldr     x10, =__primary_stack_size
            ldr     x10, [x10]
            add     sp, x9, x10
            b       main
    "#
}
