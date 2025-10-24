<!--
     Copyright 2024, Colias Group, LLC

     SPDX-License-Identifier: CC-BY-SA-4.0
-->

# `#![feature(sync_unsafe_cell)]`

Use `SyncUnsafeCell`:
- `ImmediateSyncOnceCell`
- `ImmutableCell`
- `OneRefCell`
- `sel4_static_heap::StaticHeap`
- `sel4_stack::Stack`
- `sel4_initialize_tls::StaticTlsAllocation`

# `#![never_type]`

Replace:
- `sel4_root_task::Never`
- `sel4_root_task_with_std::Never`

Simplify type of `sel4::init_thread::suspend_self`

# `#![feature(array_try_from_fn)]`

Improve implementation of `sel4_capdl_initializer_core::hold_slots::HoldSlots::new` (see git blame)

# `#![feature(btree_cursors)]`

Improve implementation of `sel4_async_time::timer_queue` (see comment in module)

Improve implementation of `sel4_bounce_buffer_allocator::basic` (see comment in module)

# `#![feature(allocator_api)]` and `#![feature(btreemap_alloc)]`

Improve API of `sel4_bounce_buffer_allocator::basic` (see comment in module)
