//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::array;
use core::borrow::BorrowMut;
use core::ops::Range;
use core::result::Result as CoreResult;
use core::slice;

use rkyv::Archive;
use rkyv::ops::ArchivedRange;

#[allow(unused_imports)]
use log::{debug, error, info, trace};

use sel4::{
    CapRights, CapTypeForFrameObjectOfFixedSize, cap_type,
    init_thread::{self, Slot},
};
use sel4_capdl_initializer_types::*;

use crate::buffers::{InitializerBuffers, PerObjectBuffer};
use crate::cslot_allocator::CSlotAllocator;
use crate::error::CapDLInitializerError;
use crate::hold_slots::HoldSlots;
use crate::memory::{CopyAddrs, get_user_image_frame_slot};

type Result<T> = CoreResult<T, CapDLInitializerError>;

pub struct Initializer<'a, B> {
    bootinfo: &'a sel4::BootInfoPtr,
    user_image_bounds: Range<usize>,
    copy_addrs: CopyAddrs,
    spec: &'a <SpecForInitializer as Archive>::Archived,
    embedded_frames_base_addr: usize,
    cslot_allocator: &'a mut CSlotAllocator,
    buffers: &'a mut InitializerBuffers<B>,
}

impl<'a, B: BorrowMut<[PerObjectBuffer]>> Initializer<'a, B> {
    pub fn initialize(
        bootinfo: &sel4::BootInfoPtr,
        user_image_bounds: Range<usize>,
        spec: &'a <SpecForInitializer as Archive>::Archived,
        embedded_frames_base_addr: usize,
        buffers: &mut InitializerBuffers<B>,
    ) -> ! {
        info!("Starting CapDL initializer");

        let copy_addrs = CopyAddrs::init(bootinfo, &user_image_bounds).unwrap();

        let mut cslot_allocator = CSlotAllocator::new(bootinfo.empty().range());

        Initializer {
            bootinfo,
            user_image_bounds,
            copy_addrs,
            spec,
            embedded_frames_base_addr,
            cslot_allocator: &mut cslot_allocator,
            buffers,
        }
        .run()
        .unwrap_or_else(|err| panic!("Error: {}", err));

        info!("CapDL initializer done, suspending");

        init_thread::suspend_self()
    }

    // // //

    fn run(&mut self) -> Result<()> {
        self.create_objects()?;

        self.init_irqs()?;
        self.init_asids()?;
        self.init_frames()?;
        self.init_vspaces()?;

        sel4::sel4_cfg_if! {
            if #[sel4_cfg(KERNEL_MCS)] {
                self.init_sched_contexts()?;
            }
        }

        self.init_tcbs()?;
        self.init_cspaces()?;

        self.start_threads()?;

        Ok(())
    }

    fn create_objects(&mut self) -> Result<()> {
        // This algorithm differs from that found in the upstream C CapDL
        // loader. In particular, this one is implemented with objects
        // specifying non-device paddrs in mind.

        debug!("Creating objects");

        // Sort untypeds by paddr
        let mut _uts_by_paddr_backing: [usize;
            sel4::sel4_cfg_usize!(MAX_NUM_BOOTINFO_UNTYPED_CAPS)] = array::from_fn(|i| i); // TODO (not a big deal) allocate in image rather than on stack
        let uts = self.bootinfo.untyped_list();
        let uts_by_paddr = {
            let arr = &mut _uts_by_paddr_backing[..uts.len()];
            arr.sort_unstable_by_key(|i| uts[*i].paddr());
            arr
        };

        // Index root objects

        let first_obj_without_paddr = self
            .root_objects()
            .partition_point(|named_obj| named_obj.object.paddr().is_some());
        let num_objs_with_paddr = first_obj_without_paddr;

        // Sanity check that all objects with a paddr attached can be allocated.
        // Currently this is only applicable to Frame objects
        let mut phys_addrs_ok = true;
        for obj_with_paddr_id in 0..first_obj_without_paddr {
            let named_obj = self.named_object(obj_with_paddr_id.into());
            let paddr_base = named_obj.object.paddr().unwrap().to_sel4();

            let blueprint = named_obj.object.blueprint().unwrap();
            let obj_size_bytes = 1 << blueprint.physical_size_bits();
            let paddr_range = paddr_base..paddr_base + obj_size_bytes;

            // Binary search for the UT that is next to the UT that might fit.
            // i.e. we are looking for the first UT that is uts[i_ut].paddr() > paddr_range.start
            let ut_after_candidate_idx = uts_by_paddr.partition_point(|&i_ut| {
                sel4::Word::try_from(uts[i_ut].paddr()).unwrap() <= paddr_range.start
            });

            if ut_after_candidate_idx == 0 {
                // Predicate returned false for the first UT, cannot allocate this object as all UTs are
                // after the object.
                phys_addrs_ok = false;
            } else {
                let candidate_ut = &uts[uts_by_paddr[ut_after_candidate_idx - 1]];
                let candidate_ut_range =
                    candidate_ut.paddr()..candidate_ut.paddr() + (1 << candidate_ut.size_bits());
                if !(sel4::Word::try_from(candidate_ut_range.start).unwrap() <= paddr_range.start
                    && sel4::Word::try_from(candidate_ut_range.end).unwrap() >= paddr_range.end)
                {
                    error!(
                        "Cannot create object '{}', with paddr {:#x}..{:#x}, size bit {} because there are no valid untypeds to cover the allocation.",
                        object_name_or_default(named_obj),
                        paddr_range.start,
                        paddr_range.end,
                        blueprint.physical_size_bits()
                    );
                    phys_addrs_ok = false;
                }
            }
        }

        if !phys_addrs_ok {
            error!("Below are the valid ranges of memory to be allocated from:");
            error!("Valid ranges outside of main memory:");
            for i_ut in uts_by_paddr.iter().filter(|i_ut| uts[**i_ut].is_device()) {
                let ut = &uts[*i_ut];
                let size_bit = ut.size_bits();
                let base = ut.paddr();
                let end = base + (1 << size_bit);
                error!("     [0x{base:0>12x}..0x{end:0>12x})");
            }
            error!("Valid ranges within main memory:");
            for i_ut in uts_by_paddr.iter().filter(|i_ut| !uts[**i_ut].is_device()) {
                let ut = &uts[*i_ut];
                let size_bit = ut.size_bits();
                let base = ut.paddr();
                let end = base + (1 << size_bit);
                error!("     [0x{base:0>12x}..0x{end:0>12x})");
            }
            panic!(
                "Encountered a spec object with physical address constraint that cannot be satisfied."
            );
        }

        let mut by_size_start: [usize; sel4::WORD_SIZE] = array::from_fn(|_| 0);
        let mut by_size_end: [usize; sel4::WORD_SIZE] = array::from_fn(|_| 0);
        {
            for obj_id in first_obj_without_paddr..self.root_objects().len() {
                let obj = &self.object(obj_id.into());
                if let Some(blueprint) = obj.blueprint() {
                    by_size_end[blueprint.physical_size_bits()] += 1;
                }
            }
            let mut acc = first_obj_without_paddr;
            for (bits, n) in by_size_end.iter_mut().enumerate().rev() {
                by_size_start[bits] = acc;
                acc += *n;
                *n = acc;
            }
        }

        // In order to allocate objects which specify paddrs, we may have to
        // allocate dummies to manipulate watermarks. We must always retain at
        // least one reference to an object allocated from an untyped, or else
        // its watermark will reset. This juggling approach is an easy way to
        // ensure that we are always holding such a reference.
        let mut hold_slots = HoldSlots::new(self.cslot_allocator, cslot_to_absolute_cptr)?;

        // Create root objects

        let mut next_obj_with_paddr = 0;
        for i_ut in uts_by_paddr.iter() {
            let ut = &uts[*i_ut];
            let ut_size_bits = ut.size_bits();
            let ut_size_bytes = 1 << ut_size_bits;
            let ut_paddr_start = ut.paddr();
            let ut_paddr_end = ut_paddr_start + ut_size_bytes;
            let mut cur_paddr = ut_paddr_start;
            trace!(
                "Allocating from untyped: {:#x}..{:#x} (size_bits = {}, device = {:?})",
                ut_paddr_start,
                ut_paddr_end,
                ut_size_bits,
                ut.is_device()
            );
            loop {
                let target = if next_obj_with_paddr < num_objs_with_paddr {
                    ut_paddr_end.min(
                        self.object(next_obj_with_paddr.into())
                            .paddr()
                            .unwrap()
                            .to_sel4()
                            .try_into()
                            .unwrap(),
                    )
                } else {
                    ut_paddr_end
                };
                let target_is_obj_with_paddr = target < ut_paddr_end;
                while cur_paddr < target {
                    let max_size_bits = usize::try_from(cur_paddr.trailing_zeros())
                        .unwrap()
                        .min((target - cur_paddr).trailing_zeros().try_into().unwrap());
                    let mut created = false;
                    if !ut.is_device() {
                        for size_bits in (0..=max_size_bits).rev() {
                            let obj_id = &mut by_size_start[size_bits];
                            // Skip embedded frames
                            while *obj_id < by_size_end[size_bits] {
                                if let ArchivedObject::Frame(obj) = self.object((*obj_id).into())
                                    && let ArchivedFrameInit::Embedded(embedded) = &obj.init
                                {
                                    self.take_cap_for_embedded_frame((*obj_id).into(), embedded)?;
                                    *obj_id += 1;
                                    continue;
                                }
                                break;
                            }
                            // Create a largest possible object that would fit
                            if *obj_id < by_size_end[size_bits] {
                                let named_obj = &self.named_object((*obj_id).into());
                                let blueprint = named_obj.object.blueprint().unwrap();
                                assert_eq!(blueprint.physical_size_bits(), size_bits);
                                trace!(
                                    "Creating kernel object: paddr=0x{:x}, size_bits={} name={:?}",
                                    cur_paddr,
                                    blueprint.physical_size_bits(),
                                    object_name_or_default(named_obj)
                                );
                                self.ut_cap(*i_ut).untyped_retype(
                                    &blueprint,
                                    &init_thread_cnode_absolute_cptr(),
                                    self.alloc_orig_cslot((*obj_id).into()).index(),
                                    1,
                                )?;
                                cur_paddr += 1 << size_bits;
                                *obj_id += 1;
                                created = true;
                                break;
                            }
                        }
                    }
                    if !created {
                        if target_is_obj_with_paddr {
                            let hold_slot = hold_slots.get_slot()?;
                            trace!(
                                "Creating dummy: paddr=0x{cur_paddr:x}, size_bits={max_size_bits}"
                            );
                            self.ut_cap(*i_ut).untyped_retype(
                                &sel4::ObjectBlueprint::Untyped {
                                    size_bits: max_size_bits,
                                },
                                &init_thread_cnode_absolute_cptr(),
                                hold_slot.index(),
                                1,
                            )?;
                            hold_slots.report_used();
                            cur_paddr += 1 << max_size_bits;
                        } else {
                            cur_paddr = target;
                        }
                    }
                }
                if target_is_obj_with_paddr {
                    let obj_id = next_obj_with_paddr;
                    let named_obj = &self.named_object(obj_id.into());
                    let blueprint = named_obj.object.blueprint().unwrap();
                    trace!(
                        "Creating device object: paddr=0x{:x}, size_bits={} name={:?}",
                        cur_paddr,
                        blueprint.physical_size_bits(),
                        object_name_or_default(named_obj)
                    );
                    self.ut_cap(*i_ut).untyped_retype(
                        &blueprint,
                        &init_thread_cnode_absolute_cptr(),
                        self.alloc_orig_cslot(obj_id.into()).index(),
                        1,
                    )?;
                    cur_paddr += 1 << blueprint.physical_size_bits();
                    next_obj_with_paddr += 1;
                } else {
                    break;
                }
            }
        }

        // Ensure that we've created every root object
        let mut oom = false;
        for bits in 0..sel4::WORD_SIZE {
            if by_size_start[bits] != by_size_end[bits] {
                oom = true;
                let shortfall = by_size_end[bits] - by_size_start[bits];
                error!(
                    "Error: ran out of untypeds for allocating objects of size bit {bits}, still need to create {shortfall} more objects."
                );
            }
        }
        if oom {
            panic!("Out of untypeds.");
        }

        // Create child objects

        for cover in self.spec.untyped_covers.iter() {
            let parent_obj_id = cover.parent;
            let parent = self.named_object(parent_obj_id);
            let parent_cptr = self.orig_cap::<cap_type::Untyped>(parent_obj_id);
            for child_obj_id in
                ArchivedObjectId::into_usize_range(&archived_range_to_range(&cover.children))
            {
                let child = &self.spec.objects[child_obj_id];
                trace!(
                    "Creating kernel object: name={:?} from {:?}",
                    object_name_or_default(child),
                    object_name_or_default(parent),
                );
                parent_cptr.untyped_retype(
                    &child.object.blueprint().unwrap(),
                    &init_thread_cnode_absolute_cptr(),
                    self.alloc_orig_cslot(child_obj_id.into()).index(),
                    1,
                )?;
            }
        }

        // Actually make the ASID pools. With the help of parse-capDL, we do
        // this in order of obj.asid_high, for verification reasons (see
        // upstream C CapDL loader).
        {
            for obj_id in self.spec.asid_slots.iter() {
                let ut = self.orig_cap(*obj_id);
                let slot = self.cslot_alloc_or_panic();
                init_thread::slot::ASID_CONTROL
                    .cap()
                    .asid_control_make_pool(ut, &cslot_to_absolute_cptr(slot))?;
                self.set_orig_cslot(*obj_id, slot);
            }
        }

        // Create IrqHandler caps
        {
            for ArchivedIrqEntry { irq, handler } in self.spec.irqs.iter() {
                let slot = self.cslot_alloc_or_panic();
                sel4::sel4_cfg_wrap_match! {
                    match self.object(*handler) {
                        ArchivedObject::Irq(_) => {
                            init_thread::slot::IRQ_CONTROL.cap()
                                .irq_control_get(irq.to_sel4(), &cslot_to_absolute_cptr(slot))?;
                        }
                        #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32))]
                        ArchivedObject::ArmIrq(obj) => {
                            sel4::sel4_cfg_if! {
                                if #[sel4_cfg(MAX_NUM_NODES = "1")] {
                                    init_thread::slot::IRQ_CONTROL.cap().irq_control_get_trigger(
                                        irq.to_sel4(),
                                        obj.extra.trigger != 0,
                                        &cslot_to_absolute_cptr(slot),
                                    )?;
                                } else {
                                    init_thread::slot::IRQ_CONTROL.cap().irq_control_get_trigger_core(
                                        irq.to_sel4(),
                                        obj.extra.trigger != 0,
                                        obj.extra.target.to_sel4(),
                                        &cslot_to_absolute_cptr(slot),
                                    )?;
                                }
                            }
                        }
                        #[sel4_cfg(ARCH_X86_64)]
                        ArchivedObject::IrqMsi(obj) => {
                            init_thread::slot::IRQ_CONTROL.cap().irq_control_get_msi(
                                obj.extra.pci_bus.to_sel4(),
                                obj.extra.pci_dev.to_sel4(),
                                obj.extra.pci_func.to_sel4(),
                                obj.extra.handle.to_sel4(),
                                irq.to_sel4(),
                                &cslot_to_absolute_cptr(slot),
                            )?;
                        }
                        #[sel4_cfg(ARCH_X86_64)]
                        ArchivedObject::IrqIOApic(obj) => {
                            init_thread::slot::IRQ_CONTROL.cap().irq_control_get_ioapic(
                                obj.extra.ioapic.to_sel4(),
                                obj.extra.pin.to_sel4(),
                                obj.extra.level.to_sel4(),
                                obj.extra.polarity.to_sel4(),
                                irq.to_sel4(),
                                &cslot_to_absolute_cptr(slot),
                            )?;
                        }
                        #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))]
                        ArchivedObject::RiscvIrq(obj) => {
                            init_thread::slot::IRQ_CONTROL.cap().irq_control_get_trigger(
                                irq.to_sel4(),
                                obj.extra.trigger != 0,
                                &cslot_to_absolute_cptr(slot),
                            )?;
                        }
                        _ => {
                            panic!();
                        }
                    }
                }
                self.set_orig_cslot(*handler, slot);
            }
        }

        // Create IOPort caps
        sel4::sel4_cfg_if! {
            if #[sel4_cfg(ARCH_X86_64)] {
                {
                    let ioports = self
                        .filter_objects::<object::ArchivedIOPorts>()
                        .map(|(obj_id, obj)| (obj_id, obj.start_port, obj.end_port));

                    for (obj_id, start_port, end_port) in ioports {
                        let slot = self.cslot_alloc_or_panic();
                        init_thread::slot::IO_PORT_CONTROL
                            .cap()
                            .ioport_control_issue(start_port.to_sel4(), end_port.to_sel4(), &cslot_to_absolute_cptr(slot))?;
                        self.set_orig_cslot(obj_id, slot);
                    }
                }
            }
        }

        // Set initial ARM SMC cap
        sel4::sel4_cfg_if! {
            if #[sel4_cfg(all(ARCH_AARCH64, ALLOW_SMC_CALLS))] {
                let arm_smc_maybe = self
                    .spec
                    .objects()
                    .enumerate()
                    .find(|(_, obj)| {
                        matches!(obj, Object::ArmSmc)
                    });
                if let Some((arm_smc_obj_id, _)) = arm_smc_maybe {
                    let arm_smc_slot_idx = init_thread::slot::SMC.index();
                    self.set_orig_cslot(arm_smc_obj_id, Slot::from_index(arm_smc_slot_idx));
                }
            }
        }

        Ok(())
    }

    fn take_cap_for_embedded_frame(
        &mut self,
        obj_id: ArchivedObjectId,
        frame_index: &ArchivedEmbeddedFrameIndex,
    ) -> Result<()> {
        let frame_addr = self.embedded_frames_base_addr
            + usize::try_from(frame_index.index).unwrap()
                * cap_type::Granule::FRAME_OBJECT_TYPE.bytes();
        let slot = get_user_image_frame_slot(self.bootinfo, &self.user_image_bounds, frame_addr);
        self.set_orig_cslot(obj_id, slot.upcast());
        Ok(())
    }

    fn init_irqs(&mut self) -> Result<()> {
        debug!("Initializing IRQs");

        let irq_notifications = self
            .filter_objects::<object::ArchivedIrq>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let arm_irq_notifications = self
            .filter_objects::<object::ArchivedArmIrq>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let msi_irq_notifications = self
            .filter_objects::<object::ArchivedIrqMsi>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let ioapic_irq_notifications = self
            .filter_objects::<object::ArchivedIrqIOApic>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let riscv_irq_notifications = self
            .filter_objects::<object::ArchivedRiscvIrq>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));

        let all_irq_notifications = irq_notifications
            .chain(arm_irq_notifications)
            .chain(msi_irq_notifications)
            .chain(ioapic_irq_notifications)
            .chain(riscv_irq_notifications);
        for (obj_id, notification) in all_irq_notifications {
            let irq_handler = self.orig_cap::<cap_type::IrqHandler>(obj_id);
            if let Some(logical_nfn_cap) = notification {
                let nfn = match logical_nfn_cap.badge.to_sel4() {
                    0 => self.orig_cap(logical_nfn_cap.object),
                    badge => {
                        let orig_cptr = self.orig_absolute_cptr(logical_nfn_cap.object);
                        let slot = self.cslot_alloc_or_panic();
                        let cptr = cslot_to_absolute_cptr(slot);
                        cptr.mint(&orig_cptr, CapRights::all(), badge)?;
                        slot.cap().downcast()
                    }
                };
                irq_handler.irq_handler_set_notification(nfn)?;
            }
        }
        Ok(())
    }

    fn init_asids(&self) -> Result<()> {
        debug!("Initializing ASIDs");
        for (obj_id, _obj) in
            self.filter_objects_with::<object::ArchivedPageTable>(|obj| obj.is_root)
        {
            let pgd = self.orig_cap::<cap_type::VSpace>(obj_id);
            init_thread::slot::ASID_POOL.cap().asid_pool_assign(pgd)?;
        }
        Ok(())
    }

    fn init_frames(&mut self) -> Result<()> {
        debug!("Initializing Frames");
        for (obj_id, obj) in self.filter_objects::<object::ArchivedFrame<_>>() {
            // TODO make more platform-agnostic
            if let ArchivedFrameInit::Fill(fill) = &obj.init
                && !fill.entries.is_empty()
            {
                let frame_object_type =
                    sel4::FrameObjectType::from_bits(obj.size_bits.into()).unwrap();
                let frame = self.orig_cap::<cap_type::UnspecifiedPage>(obj_id);
                self.fill_frame(frame, frame_object_type, &fill.entries)?;
            }
        }
        Ok(())
    }

    fn fill_frame(
        &self,
        frame: sel4::Cap<cap_type::UnspecifiedPage>,
        frame_object_type: sel4::FrameObjectType,
        fill: &[ArchivedFillEntry<Content>],
    ) -> Result<()> {
        frame.frame_map(
            init_thread::slot::VSPACE.cap(),
            self.copy_addrs.select(frame_object_type),
            CapRights::read_write(),
            vm_attributes_from_whether_cached_and_exec(true, false),
        )?;
        for entry in fill.iter() {
            let range = try_into_usize_range(&archived_range_to_range(&entry.range)).unwrap();
            let offset = range.start;
            let length = range.len();
            assert!(range.end <= frame_object_type.bytes());
            let dst_frame = self.copy_addrs.select(frame_object_type) as *mut u8;
            let dst = unsafe { slice::from_raw_parts_mut(dst_frame.add(offset), length) };
            match &entry.content {
                ArchivedFillEntryContent::Data(content_data) => {
                    content_data.copy_out(dst);
                }
                ArchivedFillEntryContent::BootInfo(content_bootinfo) => {
                    for extra in self.bootinfo.extra() {
                        if extra.id == content_bootinfo.id.to_sel4() {
                            let n =
                                dst.len()
                                    .min(extra.content_with_header().len().saturating_sub(
                                        content_bootinfo.offset.to_native().try_into().unwrap(),
                                    ));
                            if n > 0 {
                                let offset =
                                    content_bootinfo.offset.to_native().try_into().unwrap();
                                dst[..n].copy_from_slice(
                                    &extra.content_with_header()[offset..(offset + n)],
                                );
                            }
                        }
                    }
                }
            }
        }
        frame.frame_unmap()?;
        Ok(())
    }

    fn init_vspaces(&mut self) -> Result<()> {
        debug!("Initializing VSpaces");
        for (obj_id, obj) in
            self.filter_objects_with::<object::ArchivedPageTable>(|obj| obj.is_root)
        {
            let vspace = self.orig_cap::<cap_type::VSpace>(obj_id);
            let root_level = obj.level.unwrap_or(0).into();
            self.init_vspace(vspace, root_level, 0, obj)?;
        }
        Ok(())
    }

    fn init_vspace(
        &mut self,
        vspace: sel4::cap::VSpace,
        level: usize,
        vaddr: usize,
        obj: &object::ArchivedPageTable,
    ) -> Result<()> {
        for (i, entry) in obj.entries() {
            let vaddr = vaddr + (usize::from(i) << sel4::vspace_levels::step_bits(level));
            match entry {
                PageTableEntry::Frame(cap) => {
                    let frame = self.orig_cap::<cap_type::UnspecifiedPage>(cap.object);
                    let rights = cap.rights.to_sel4();
                    self.copy(frame)?
                        .frame_map(vspace, vaddr, rights, cap.vm_attributes())?;
                }
                PageTableEntry::PageTable(cap) => {
                    self.orig_cap::<cap_type::UnspecifiedIntermediateTranslationTable>(cap.object)
                        .generic_intermediate_translation_table_map(
                            sel4::TranslationTableObjectType::from_level(level + 1).unwrap(),
                            vspace,
                            vaddr,
                            cap.vm_attributes(),
                        )?;
                    let obj = self.lookup_object::<object::ArchivedPageTable>(cap.object);
                    self.init_vspace(vspace, level + 1, vaddr, obj)?;
                }
            }
        }
        Ok(())
    }

    #[sel4::sel4_cfg(KERNEL_MCS)]
    fn init_sched_contexts(&self) -> Result<()> {
        debug!("Initializing scheduling contexts");
        for (obj_id, _obj) in self.filter_objects::<object::ArchivedSchedContext>() {
            self.init_sched_context(obj_id, 0)?;
        }
        Ok(())
    }

    #[sel4::sel4_cfg(KERNEL_MCS)]
    fn init_sched_context(&self, obj_id: ArchivedObjectId, affinity: usize) -> Result<()> {
        let obj = self.lookup_object::<object::ArchivedSchedContext>(obj_id);
        let sched_context = self.orig_cap::<cap_type::SchedContext>(obj_id);
        self.bootinfo
            .sched_control()
            .index(affinity)
            .cap()
            .sched_control_configure_flags(
                sched_context,
                obj.extra.budget.to_native(),
                obj.extra.period.to_native(),
                0,
                obj.extra.badge.to_sel4(),
                0,
            )?;
        Ok(())
    }

    fn init_tcbs(&mut self) -> Result<()> {
        debug!("Initializing TCBs");

        for (obj_id, obj) in self.filter_objects::<object::ArchivedTcb>() {
            let tcb = self.orig_cap::<cap_type::Tcb>(obj_id);

            if let Some(bound_notification) = obj.bound_notification() {
                let bound_notification =
                    self.orig_cap::<cap_type::Notification>(bound_notification.object);
                tcb.tcb_bind_notification(bound_notification)?;
            }

            sel4::sel4_cfg_if! {
                if #[sel4_cfg(any(all(ARCH_AARCH64, ARM_HYPERVISOR_SUPPORT), all(ARCH_X86_64, VTX)))] {
                    if let Some(vcpu) = obj.vcpu() {
                        let vcpu = self.orig_cap::<cap_type::VCpu>(vcpu.object);
                        vcpu.vcpu_set_tcb(tcb)?;
                    }
                }
            }

            {
                let cspace = self.orig_cap(obj.cspace().object);
                let cspace_root_data = sel4::CNodeCapData::new(
                    obj.cspace().guard.to_sel4(),
                    obj.cspace().guard_size.into(),
                );
                let vspace = self.orig_cap(obj.vspace().object);
                let ipc_buffer_addr = obj.extra.ipc_buffer_addr.to_sel4();
                let ipc_buffer_frame = self.orig_cap(obj.ipc_buffer().object);

                let authority = init_thread::slot::TCB.cap();
                let max_prio = obj.extra.max_prio.into();
                let prio = obj.extra.prio.into();

                #[allow(unused_variables)]
                let affinity = obj.extra.affinity.to_sel4();

                sel4::sel4_cfg_if! {
                    if #[sel4_cfg(KERNEL_MCS)] {
                        if let Some(sched_context_cap) = obj.sc() {
                            self.init_sched_context(sched_context_cap.object, affinity.try_into().unwrap())?;
                        }

                        tcb.tcb_configure(
                            cspace,
                            cspace_root_data,
                            vspace,
                            ipc_buffer_addr,
                            ipc_buffer_frame,
                        )?;

                        let sc = match obj.sc() {
                            None => init_thread::slot::NULL.cap().cast::<cap_type::SchedContext>(),
                            Some(cap) => self.orig_cap::<cap_type::SchedContext>(cap.object),
                        };

                        let fault_ep = match obj.mcs_fault_ep() {
                            None => init_thread::slot::NULL.cap().cast::<cap_type::Endpoint>(),
                            Some(cap) => {
                                let orig = self.orig_cap::<cap_type::Endpoint>(cap.object);
                                let badge = cap.badge.to_sel4();
                                let rights = cap.rights.to_sel4();
                                if badge == 0 && rights == CapRights::all() {
                                    orig
                                } else {
                                    let src = init_thread::slot::CNODE.cap().absolute_cptr(orig);
                                    let new = self.cslot_alloc_or_panic().cap();
                                    let dst = init_thread::slot::CNODE.cap().absolute_cptr(new);
                                    dst.mint(&src, rights, badge)?;
                                    new.cast()
                                }
                            },
                        };

                        let temp_fault_ep = match obj.temp_fault_ep() {
                            None => init_thread::slot::NULL.cap().cast::<cap_type::Endpoint>(),
                            Some(cap) => {
                                assert_eq!(cap.badge.to_sel4(), 0); // HACK
                                self.orig_cap::<cap_type::Endpoint>(cap.object)
                            },
                        };

                        tcb.tcb_set_sched_params(
                            authority,
                            max_prio,
                            prio,
                            sc,
                            fault_ep,
                        )?;

                        tcb.tcb_set_timeout_endpoint(temp_fault_ep)?;
                    } else {
                        let fault_ep = sel4::CPtr::from_bits(obj.extra.master_fault_ep.as_ref().unwrap().to_sel4());

                        tcb.tcb_configure(
                            fault_ep,
                            cspace,
                            cspace_root_data,
                            vspace,
                            ipc_buffer_addr,
                            ipc_buffer_frame,
                        )?;

                        tcb.tcb_set_sched_params(
                            authority,
                            max_prio,
                            prio,
                        )?;

                        sel4::sel4_cfg_if! {
                            if #[sel4_cfg(not(MAX_NUM_NODES = "1"))] {
                                tcb.tcb_set_affinity(affinity)?;
                            }
                        }
                    }
                }
            }

            {
                let mut regs = sel4::UserContext::default();
                *regs.pc_mut() = obj.extra.ip.to_sel4();
                *regs.sp_mut() = obj.extra.sp.to_sel4();
                for (i, value) in obj.extra.gprs.iter().enumerate() {
                    *regs.c_param_mut(i) = value.to_sel4();
                }
                tcb.tcb_write_all_registers(false, &mut regs)?;
            }

            sel4::sel4_cfg_if! {
                if #[sel4_cfg(DEBUG_BUILD)] {
                    if let Some(name) = object_name(self.named_object(obj_id)) {
                        tcb.debug_name(name.as_bytes());
                    }
                }
            }
        }
        Ok(())
    }

    fn init_cspaces(&self) -> Result<()> {
        debug!("Initializing CSpaces");

        for (obj_id, obj) in self.filter_objects::<object::ArchivedCNode>() {
            let cnode = self.orig_cap::<cap_type::CNode>(obj_id);
            for entry in obj.slots() {
                let badge = entry.cap.badge();
                let rights = entry.cap.rights().unwrap_or(CapRights::all());
                let src = init_thread::slot::CNODE
                    .cap()
                    .absolute_cptr(self.orig_cap::<cap_type::Unspecified>(entry.cap.obj()));
                let dst = cnode.absolute_cptr_from_bits_with_depth(
                    usize::from(entry.slot).try_into().unwrap(),
                    obj.size_bits.into(),
                );
                match badge {
                    None => dst.copy(&src, rights),
                    Some(badge) => dst.mint(&src, rights, badge),
                }?;
            }
        }
        Ok(())
    }

    fn start_threads(&self) -> Result<()> {
        debug!("Starting threads");
        for (obj_id, obj) in self.filter_objects::<object::ArchivedTcb>() {
            let tcb = self.orig_cap::<cap_type::Tcb>(obj_id);
            if obj.extra.resume {
                tcb.tcb_resume()?;
            }
        }
        Ok(())
    }

    //

    fn named_object(&self, obj_id: ArchivedObjectId) -> &'a ArchivedNamedObject<FrameInit> {
        &self.spec.objects[usize::from(obj_id)]
    }

    fn object(&self, obj_id: ArchivedObjectId) -> &'a ArchivedObject<FrameInit> {
        &self.named_object(obj_id).object
    }

    fn root_objects(&self) -> &[ArchivedNamedObject<FrameInit>] {
        &self.spec.objects
            [ArchivedObjectId::into_usize_range(&archived_range_to_range(&self.spec.root_objects))]
    }

    fn named_objects(&self) -> impl Iterator<Item = &'a ArchivedNamedObject<FrameInit>> + 'a {
        self.spec.objects.iter()
    }

    fn objects(&self) -> impl Iterator<Item = &'a ArchivedObject<FrameInit>> + 'a {
        self.named_objects()
            .map(|named_object| &named_object.object)
    }

    fn filter_objects<O: IsArchivedObject<FrameInit> + 'a>(
        &self,
    ) -> impl Iterator<Item = (ArchivedObjectId, &'a O)> + 'a {
        self.objects()
            .enumerate()
            .filter_map(|(obj_id, obj)| Some((obj_id.into(), obj.as_()?)))
    }

    fn filter_objects_with<O: IsArchivedObject<FrameInit> + 'a>(
        &self,
        f: impl Fn(&'a O) -> bool + 'a,
    ) -> impl Iterator<Item = (ArchivedObjectId, &'a O)> + 'a {
        self.filter_objects().filter(move |(_, obj)| (f)(obj))
    }

    fn lookup_object<O: IsArchivedObject<FrameInit> + 'a>(
        &self,
        obj_id: ArchivedObjectId,
    ) -> &'a O {
        self.object(obj_id).as_().unwrap()
    }

    //

    fn copy<U: sel4::CapType>(&mut self, cap: sel4::Cap<U>) -> Result<sel4::Cap<U>> {
        let slot = self.cslot_alloc_or_panic();
        let src = init_thread::slot::CNODE.cap().absolute_cptr(cap);
        cslot_to_absolute_cptr(slot).copy(&src, CapRights::all())?;
        Ok(slot.cap().downcast())
    }

    //

    fn cslot_alloc_or_panic(&mut self) -> Slot {
        self.cslot_allocator.alloc_or_panic()
    }

    fn set_orig_cslot(&mut self, obj_id: ArchivedObjectId, slot: Slot) {
        self.buffers.per_obj_mut()[usize::from(obj_id)].orig_slot = Some(slot);
    }

    fn orig_cslot(&self, obj_id: ArchivedObjectId) -> Slot {
        self.buffers.per_obj()[usize::from(obj_id)]
            .orig_slot
            .unwrap()
    }

    fn alloc_orig_cslot(&mut self, obj_id: ArchivedObjectId) -> Slot {
        let slot = self.cslot_alloc_or_panic();
        self.set_orig_cslot(obj_id, slot);
        slot
    }

    fn orig_cap<U: sel4::CapType>(&self, obj_id: ArchivedObjectId) -> sel4::Cap<U> {
        self.orig_cslot(obj_id).cap().downcast()
    }

    fn orig_absolute_cptr(&self, obj_id: ArchivedObjectId) -> sel4::AbsoluteCPtr {
        cslot_to_absolute_cptr(self.orig_cslot(obj_id))
    }

    fn ut_cap(&self, ut_index: usize) -> sel4::cap::Untyped {
        self.bootinfo.untyped().index(ut_index).cap()
    }
}

fn cslot_to_absolute_cptr(slot: Slot) -> sel4::AbsoluteCPtr {
    init_thread::slot::CNODE.cap().absolute_cptr(slot.cptr())
}

fn init_thread_cnode_absolute_cptr() -> sel4::AbsoluteCPtr {
    init_thread::slot::CNODE.cap().absolute_cptr_for_self()
}

fn object_name(named_obj: &ArchivedNamedObject<FrameInit>) -> Option<&str> {
    named_obj.name.as_ref().map(|x| x.as_str())
}

fn object_name_or_default(named_obj: &ArchivedNamedObject<FrameInit>) -> &str {
    object_name(named_obj).unwrap_or("<unnamed>")
}

fn archived_range_to_range<T: Copy>(archived_range: &ArchivedRange<T>) -> Range<T> {
    archived_range.start..archived_range.end
}

fn try_into_usize_range<T: TryInto<usize> + Copy>(
    range: &Range<T>,
) -> CoreResult<Range<usize>, <T as TryInto<usize>>::Error> {
    Ok(range.start.try_into()?..range.end.try_into()?)
}
