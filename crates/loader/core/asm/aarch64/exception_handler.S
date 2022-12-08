#include "macros.h"

.extern exception_handler
.extern exception_register_state

.macro ventry id
.align 7
    /* push some temp registers on the stack */
    stp     x2, x3, [sp, #-16]
    adrp    x2, exception_register_state
    stp     x0, x1, [x2]
    mov     x0, x2
    ldp     x2, x3, [sp, #-16]
    stp     x2, x3, [x0, #16 * 1]
    stp     x4, x5, [x0, #16 * 2]
    stp     x6, x7, [x0, #16 * 3]
    stp     x8, x9, [x0, #16 * 4]
    stp     x10, x11, [x0, #16 * 5]
    stp     x12, x13, [x0, #16 * 6]
    stp     x14, x15, [x0, #16 * 7]
    stp     x16, x17, [x0, #16 * 8]
    stp     x18, x19, [x0, #16 * 9]
    stp     x20, x21, [x0, #16 * 10]
    stp     x22, x23, [x0, #16 * 11]
    stp     x24, x25, [x0, #16 * 12]
    stp     x26, x27, [x0, #16 * 13]
    stp     x28, x29, [x0, #16 * 14]
    mov     x0, \id
    b       exception_handler
.endm

.align 12
BEGIN_FUNC(arm_vector_table)
    ventry  #0
    ventry  #1
    ventry  #2
    ventry  #3
    ventry  #4
    ventry  #5
    ventry  #6
    ventry  #7
    ventry  #8
    ventry  #9
    ventry  #10
    ventry  #11
    ventry  #12
    ventry  #13
    ventry  #14
    ventry  #15
END_FUNC(arm_vector_table)
