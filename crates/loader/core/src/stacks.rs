use core::sync::Exclusive;

use crate::NUM_SECONDARY_CORES;

#[repr(C, align(16))]
struct Stack<const N: usize>([u8; N]);

const PRIMARY_STACK_SIZE: usize = 4096 * 3;

static mut PRIMARY_STACK: Stack<PRIMARY_STACK_SIZE> = Stack([0; PRIMARY_STACK_SIZE]);

#[no_mangle]
static __primary_stack_bottom: Exclusive<*const u8> = Exclusive::new(unsafe { PRIMARY_STACK.0.as_ptr_range().end });

const SECONDARY_STACK_SIZE: usize = 4096 * 2;
const SECONDARY_STACKS_SIZE: usize = SECONDARY_STACK_SIZE * NUM_SECONDARY_CORES;

static SECONDARY_STACKS: Stack<SECONDARY_STACKS_SIZE> = Stack([0; SECONDARY_STACKS_SIZE]);

pub(crate) fn get_secondary_core_initial_stack_pointer(i: usize) -> usize {
    unsafe {
        SECONDARY_STACKS
            .0
            .as_ptr()
            .offset(((i + 1) * SECONDARY_STACK_SIZE).try_into().unwrap())
            .to_bits()
    }
}
