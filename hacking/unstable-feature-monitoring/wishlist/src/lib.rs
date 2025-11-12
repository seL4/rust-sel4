//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//
#![feature(custom_inner_attributes)]
#![rustfmt::skip]

#![feature(exclusive_wrapper)]
#![feature(sync_unsafe_cell)]
#![feature(variant_count)]

// For operations on *(const|mut) [T]:
//   - pointer::as_mut_ptr
//   - NonNull::as_non_null_ptr
#![feature(slice_ptr_get)]

// For sel4_microkit::Handler::Error = !
#![feature(associated_type_defaults)]

// Would enable sel4_bounce_buffer_allocator::Basic without a global heap
#![feature(allocator_api)]
#![feature(btreemap_alloc)]

// For core::arch::riscv*
#![cfg_attr(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    feature(riscv_ext_intrinsics),
)]

// For pointer::is_aligned_to
//
// For now, use:
// ```
// assert_eq!(ptr.cast::<()>().align_offset(x), 0)
// ```
// (See definitions of pointer::is_aligned_to)
#![feature(pointer_is_aligned_to)]

// Without these, the more invasive sel4_cfg_if! and sel4_cfg_wrap_match! must be used instead of
// #[sel4_cfg] and #[sel4_cfg_match] on expressions and non-inline module declarations.
// Also, proc_macro_hygiene will enable the use of sel4-mod-in-out-dir in tests-root-task-dafny-core.
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

#![feature(never_type)]
#![feature(unwrap_infallible)]

// Useful throughout, including Default for sel4_capdl_initializer_core::PerObjectBuffer
#![feature(const_trait_impl)]
#![feature(derive_const)]

// Will replace sel4_panicking::abort_unwind
#![feature(abort_unwind)]

// For global_asm! throughout
#![feature(asm_cfg)]

// To replace runtime macro crates
#![feature(macro_attr)]
#![feature(macro_derive)]
