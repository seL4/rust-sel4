//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_variables)]
#![allow(dead_code)]

use std::ops::Range;

use crate::page_tables::{
    LeafDescriptor, MkLeafArgs, RawDescriptor, Region, RegionsBuilder, Scheme, schemes,
};
use crate::platform_info::PlatformInfoForBuildSystem;

pub fn mk_loader_map(
    scheme: &Scheme,
    smp: bool,
    vaddr: u64,
    platform_info: &PlatformInfoForBuildSystem,
) -> (Vec<u8>, u64) {
    let device_range_end = match scheme {
        Scheme::AArch64 => 1 << 39,
        Scheme::AArch32 => scheme.virt_bounds().end,
        _ => panic!(),
    };

    let mut regions = RegionsBuilder::new(scheme);
    regions = regions.insert(Region::valid(
        0..device_range_end,
        mk_device_leaf_for_loader_map,
    ));
    for range in platform_info.memory.iter() {
        regions = regions.insert(Region::valid(range.clone(), move |args| {
            mk_normal_leaf_for_loader_map(smp, args)
        }));
    }

    regions.build().construct_table(scheme).embed(scheme, vaddr)
}

pub fn mk_kernel_map(
    scheme: &Scheme,
    smp: bool,
    vaddr: u64,
    kernel_virt_addr_range: Range<u64>,
    kernel_phys_to_virt_offset: u64,
) -> (Vec<u8>, u64) {
    let virt_start = kernel_virt_addr_range.start;
    let virt_end = kernel_virt_addr_range.end;
    let virt_map_end = virt_end.next_multiple_of(1 << scheme.largest_leaf_size_bits());

    let regions = RegionsBuilder::new(scheme)
        .insert(Region::valid(
            0..virt_start,
            mk_identity_leaf_for_kernel_map,
        ))
        .insert(Region::valid(virt_start..virt_map_end, move |loc| {
            mk_kernel_leaf_for_kernel_map(smp, kernel_phys_to_virt_offset, loc)
        }));

    regions.build().construct_table(scheme).embed(scheme, vaddr)
}

fn mk_normal_leaf_for_loader_map(smp: bool, loc: MkLeafArgs) -> RawDescriptor {
    match loc.scheme() {
        Scheme::AArch64 => {
            loc.identity_descriptor::<schemes::AArch64LeafDescriptor>()
                .set_access_flag(true)
                .set_attribute_index(4) // select MT_NORMAL
                .set_shareability(aarch64_normal_shareability(smp))
                .to_raw()
        }
        Scheme::AArch32 => loc
            .identity_descriptor::<schemes::AArch32LeafDescriptor>()
            .set_access_flag(true)
            .set_attributes(0b101, false, true)
            .set_shareability(true)
            .to_raw(),
        _ => panic!(),
    }
}

fn mk_device_leaf_for_loader_map(loc: MkLeafArgs) -> RawDescriptor {
    match loc.scheme() {
        Scheme::AArch64 => loc
            .identity_descriptor::<schemes::AArch64LeafDescriptor>()
            .set_access_flag(true)
            .set_attribute_index(0)
            .to_raw(),
        Scheme::AArch32 => loc
            .identity_descriptor::<schemes::AArch32LeafDescriptor>()
            .set_access_flag(true)
            .to_raw(),
        _ => panic!(),
    }
}

fn mk_identity_leaf_for_kernel_map(loc: MkLeafArgs) -> RawDescriptor {
    match loc.scheme() {
        Scheme::AArch64 => loc
            .identity_descriptor::<schemes::AArch64LeafDescriptor>()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
            .to_raw(),
        Scheme::AArch32 => loc
            .identity_descriptor::<schemes::AArch32LeafDescriptor>()
            .set_access_flag(true)
            .to_raw(),
        Scheme::RiscVSv39 | Scheme::RiscVSv32 => loc
            .identity_descriptor::<schemes::RiscVLeafDescriptor>()
            .to_raw(),
    }
}

fn mk_kernel_leaf_for_kernel_map(
    smp: bool,
    phys_to_virt_offset: u64,
    loc: MkLeafArgs,
) -> RawDescriptor {
    let f = |vaddr: u64| vaddr.wrapping_sub(phys_to_virt_offset);
    match loc.scheme() {
        Scheme::AArch64 => loc
            .descriptor::<schemes::AArch64LeafDescriptor>(f)
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(aarch64_normal_shareability(smp))
            .to_raw(),
        Scheme::AArch32 => loc
            .descriptor::<schemes::AArch32LeafDescriptor>(f)
            .set_access_flag(true)
            .set_shareability(true)
            .to_raw(),
        Scheme::RiscVSv39 | Scheme::RiscVSv32 => {
            loc.descriptor::<schemes::RiscVLeafDescriptor>(f).to_raw()
        }
    }
}

fn aarch64_normal_shareability(smp: bool) -> u64 {
    if smp { 0b11 } else { 0b00 }
}
