//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![feature(cell_update)]
#![feature(exclusive_wrapper)]
#![feature(exposed_provenance)]
#![feature(extract_if)]
#![feature(let_chains)]
#![feature(strict_provenance)]
#![feature(sync_unsafe_cell)]
#![feature(variant_count)]

// For operations on *(const|mut) [T]:
//   - pointer::as_mut_ptr
//   - NonNull::as_non_null_ptr
#![feature(slice_ptr_get)]

// For sel4_microkit::Handler::Error = !
#![feature(associated_type_defaults)]

// Would greatly simplify sel4_render_elf_with_data::FileHeaderExt
#![feature(associated_type_bounds)]

// Would enable sel4_bounce_buffer_allocator::Basic without a global heap
#![feature(allocator_api)]
#![feature(btreemap_alloc)]

// For core::arch::riscv*
#![cfg_attr(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    feature(riscv_ext_intrinsics),
)]

// For pointer::is_aligned_to?
//
// For now, use:
// ```
// assert_eq!(ptr.cast::<()>().align_offset(x), 0)
// ```
// (See definitions of pointer::is_aligned_to?)
#![feature(pointer_is_aligned_to)]

// Without these, the more invasive sel4_cfg_if! and sel4_cfg_wrap_match! must be used instead of
// #[sel4_cfg] and #[sel4_cfg_match] on expressions and non-inline module declarations.
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

#![feature(never_type)]
#![feature(unwrap_infallible)]
