//
// Copyright 2026, UNSW
//
// SPDX-License-Identifier: BSD-2-Clause
//

/* seL4 does not enforce a fixed io address space layout. In particular the maximum virtual address
 width is determined at run-time and reported in the bootinfo. In addition, support for different
 page sizes i.e. 4KiB and 2MiB is also only determined at runtime. Finally, the set of valid
 domain IDs required when minting an IOSpace capability is only determined at runtime.
 Thus to enable generation of a CAPDL spec we make simplifying assumptions defined in here.
*/

/// The number of x86 VT-d IOPT levels represented directly in the capDL spec.
/// seL4 may report a wider runtime IOVA space in bootinfo. In that case the
/// initializer maps zero-address prefix IOPTs above this lower 39-bit tree.
pub const CAPDL_NUM_IOPT_LEVELS: usize = 3;

/// For clients to build a consistent spec we define the maximum IOVA that
/// can be supported, independent of what seL4 reports at run time.
pub const CAPDL_MAX_IOVA: u64 = (1_u64 << 39) - 1;

/// The maximum x86 VT-d IOPT depth seL4 can report for a 64-bit IOVA space.
pub const MAX_RUNTIME_NUM_LEVELS: usize = 6;

/// Number of spare IOPTs the capDL spec reserves for runtime prefix levels.
pub const SPARE_NUM_LEVELS: usize = MAX_RUNTIME_NUM_LEVELS - CAPDL_NUM_IOPT_LEVELS;

/// The reserved slot on the IOSpace object that holds the root IOPT.
pub const IOSPACE_ROOT_IOPT_SLOT: usize = 0;
