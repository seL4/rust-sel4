#![no_std]
#![feature(array_try_from_fn)]
#![feature(const_trait_impl)]
#![feature(int_roundings)]
#![feature(never_type)]
#![feature(pointer_is_aligned)]
#![feature(proc_macro_hygiene)]
#![feature(slice_ptr_len)]
#![feature(stmt_expr_attributes)]
#![feature(strict_provenance)]

use core::array;
use core::borrow::BorrowMut;
use core::ops::Range;
use core::ptr;
use core::result;
use core::slice;
use core::sync::atomic::{self, Ordering};

#[allow(unused_imports)]
use log::{debug, info, trace};

use sel4::{
    cap_type, AbsoluteCPtr, BootInfo, CNodeCapData, CPtr, CapRights, CapType, FrameSize, FrameType,
    InitCSpaceSlot, LocalCPtr, ObjectBlueprint, Untyped, UserContext,
};
use sel4_capdl_initializer_types::*;

mod arch;
mod buffers;
mod cslot_allocator;
mod error;
mod hold_slots;
mod memory;

use arch::frame_types;
pub use buffers::{InitializerBuffers, PerObjectBuffer};
use cslot_allocator::{CSlotAllocator, CSlotAllocatorError};
pub use error::CapDLInitializerError;
use hold_slots::HoldSlots;
use memory::{get_user_image_frame_slot, init_copy_addrs};

type Result<T> = result::Result<T, CapDLInitializerError>;

pub struct Initializer<'a, N: ObjectName, D: Content, M: GetEmbeddedFrame, B> {
    bootinfo: &'a BootInfo,
    user_image_bounds: Range<usize>,
    small_frame_copy_addr: usize,
    large_frame_copy_addr: usize,
    spec_with_sources: &'a SpecWithSources<'a, N, D, M>,
    cslot_allocator: &'a mut CSlotAllocator,
    buffers: &'a mut InitializerBuffers<B>,
}

impl<'a, N: ObjectName, D: Content, M: GetEmbeddedFrame, B: BorrowMut<[PerObjectBuffer]>>
    Initializer<'a, N, D, M, B>
{
    pub fn initialize(
        bootinfo: &BootInfo,
        user_image_bounds: Range<usize>,
        spec_with_sources: &SpecWithSources<N, D, M>,
        buffers: &mut InitializerBuffers<B>,
    ) -> Result<!> {
        info!("Starting CapDL initializer");

        let (small_frame_copy_addr, large_frame_copy_addr) =
            init_copy_addrs(bootinfo, &user_image_bounds)?;

        let mut cslot_allocator = CSlotAllocator::new(bootinfo.empty());

        Initializer {
            bootinfo,
            user_image_bounds,
            small_frame_copy_addr,
            large_frame_copy_addr,
            spec_with_sources,
            cslot_allocator: &mut cslot_allocator,
            buffers,
        }
        .run()
    }

    fn spec(&self) -> &'a Spec<'a, N, D, M> {
        &self.spec_with_sources.spec
    }

    fn object_name(&self, indirect: &'a N) -> Option<&'a str> {
        indirect.object_name(self.spec_with_sources.object_name_source)
    }

    // // //

    fn run(&mut self) -> Result<!> {
        self.create_objects()?;

        self.init_irqs()?;
        self.init_asids()?;
        self.init_frames()?;
        self.init_vspaces()?;

        #[sel4::sel4_cfg(KERNEL_MCS)]
        self.init_sched_contexts()?;

        self.init_tcbs()?;
        self.init_cspaces()?;

        self.start_threads()?;

        info!("CapDL initializer done, suspending");

        BootInfo::init_thread_tcb().tcb_suspend()?;

        unreachable!()
    }

    fn create_objects(&mut self) -> Result<()> {
        // This algorithm differs from that found in the upstream C CapDL
        // loader. In particular, this one is implemented with objects
        // specifying non-device paddrs in mind.

        debug!("Creating objects");

        // Sort untypeds by paddr, not taking isDevice into account.
        // The kernel provides them sorted first by isDevice.
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
            .spec()
            .root_objects()
            .partition_point(|named_obj| named_obj.object.paddr().is_some());
        let num_objs_with_paddr = first_obj_without_paddr;

        let mut by_size_start: [usize; sel4::WORD_SIZE] = array::from_fn(|_| 0);
        let mut by_size_end: [usize; sel4::WORD_SIZE] = array::from_fn(|_| 0);
        {
            for obj_id in first_obj_without_paddr..self.spec().root_objects().len() {
                let obj = &self.spec().object(obj_id);
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

        // trace!("num_objs_with_paddr: {}", num_objs_with_paddr);

        // for i in 0..sel4::WORD_SIZE {
        //     trace!(
        //         "by_size[{}]: {}..{} (n = {})",
        //         i,
        //         by_size_start[i],
        //         by_size_end[i],
        //         by_size_end[i] - by_size_start[i]
        //     );
        // }

        // In order to allocate objects which specify paddrs, we may have to
        // allocate dummies to manipulate watermarks. We must always retain at
        // least one reference to an object allocated from an untyped, or else
        // its watermark will reset. This juggling approach is an easy way to
        // ensure that we are always holding such a reference.
        let mut hold_slots = HoldSlots::new(self.cslot_allocator, cslot_relative_cptr)?;

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
                    ut_paddr_end.min(self.spec().object(next_obj_with_paddr).paddr().unwrap())
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
                                if let Object::Frame(obj) = self.spec().object(*obj_id) {
                                    if let FrameInit::Embedded(embedded) = &obj.init {
                                        self.take_cap_for_embedded_frame(
                                            *obj_id,
                                            &embedded.get_embedded_frame(
                                                &self.spec_with_sources.embedded_frame_source,
                                            ),
                                        )?;
                                        *obj_id += 1;
                                        continue;
                                    }
                                }
                                break;
                            }
                            // Create a largest possible object that would fit
                            if *obj_id < by_size_end[size_bits] {
                                let named_obj = &self.spec().named_object(*obj_id);
                                let blueprint = named_obj.object.blueprint().unwrap();
                                assert_eq!(blueprint.physical_size_bits(), size_bits);
                                trace!(
                                    "Creating kernel object: paddr=0x{:x}, size_bits={} name={:?}",
                                    cur_paddr,
                                    blueprint.physical_size_bits(),
                                    self.object_name(&named_obj.name).unwrap_or("<none>")
                                );
                                self.ut_local_cptr(*i_ut).untyped_retype(
                                    &blueprint,
                                    &init_thread_cnode_relative_cptr(),
                                    self.alloc_orig_cslot(*obj_id),
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
                                "Creating dummy: paddr=0x{:x}, size_bits={}",
                                cur_paddr,
                                max_size_bits
                            );
                            self.ut_local_cptr(*i_ut).untyped_retype(
                                &ObjectBlueprint::Untyped {
                                    size_bits: max_size_bits,
                                },
                                &init_thread_cnode_relative_cptr(),
                                hold_slot,
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
                    let named_obj = &self.spec().named_object(obj_id);
                    let blueprint = named_obj.object.blueprint().unwrap();
                    trace!(
                        "Creating device object: paddr=0x{:x}, size_bits={} name={:?}",
                        cur_paddr,
                        blueprint.physical_size_bits(),
                        self.object_name(&named_obj.name).unwrap_or("<none>")
                    );
                    self.ut_local_cptr(*i_ut).untyped_retype(
                        &blueprint,
                        &init_thread_cnode_relative_cptr(),
                        self.alloc_orig_cslot(obj_id),
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
        for bits in 0..sel4::WORD_SIZE {
            assert_eq!(by_size_start[bits], by_size_end[bits], "!!! {}", bits);
        }

        // Create child objects

        for cover in self.spec().untyped_covers.iter() {
            let parent_obj_id = cover.parent;
            let parent = self.spec().named_object(parent_obj_id);
            let parent_cptr = self.orig_local_cptr::<cap_type::Untyped>(parent_obj_id);
            for child_obj_id in cover.children.clone() {
                let child = &self.spec().objects[child_obj_id];
                trace!(
                    "Creating kernel object: name={:?} from {:?}",
                    self.object_name(&child.name).unwrap_or("<none>"),
                    self.object_name(&parent.name).unwrap_or("<none>"),
                );
                parent_cptr.untyped_retype(
                    &child.object.blueprint().unwrap(),
                    &init_thread_cnode_relative_cptr(),
                    self.alloc_orig_cslot(child_obj_id),
                    1,
                )?;
            }
        }

        // Actually make the ASID pools. With the help of parse-capDL, we do
        // this in order of obj.asid_high, for verification reasons (see
        // upstream C CapDL loader).
        {
            for obj_id in self.spec().asid_slots.iter() {
                let ut = self.orig_local_cptr(*obj_id);
                let slot = self.cslot_alloc_or_panic();
                BootInfo::asid_control().asid_control_make_pool(ut, &cslot_relative_cptr(slot))?;
                self.set_orig_cslot(*obj_id, slot);
            }
        }

        // Create IRQHandler caps
        {
            for IRQEntry { irq, handler } in self.spec().irqs.iter() {
                let slot = self.cslot_alloc_or_panic();
                #[sel4::sel4_cfg_match]
                match self.spec().object(*handler) {
                    Object::IRQ(_) => {
                        BootInfo::irq_control()
                            .irq_control_get(*irq, &cslot_relative_cptr(slot))?;
                    }
                    #[sel4_cfg(any(ARCH_AARCH32, ARCH_AARCH64))]
                    Object::ArmIRQ(obj) => {
                        sel4::sel4_cfg_if! {
                            if #[cfg(MAX_NUM_NODES = "1")] {
                                BootInfo::irq_control().irq_control_get_trigger(
                                    *irq,
                                    obj.extra.trigger,
                                    &cslot_relative_cptr(slot),
                                )?;
                            } else {
                                BootInfo::irq_control().irq_control_get_trigger_core(
                                    *irq,
                                    obj.extra.trigger,
                                    obj.extra.target,
                                    &cslot_relative_cptr(slot),
                                )?;
                            }
                        }
                    }
                    _ => {
                        panic!();
                    }
                }
                self.set_orig_cslot(*handler, slot);
            }
        }

        Ok(())
    }

    fn take_cap_for_embedded_frame(
        &mut self,
        obj_id: ObjectId,
        frame: &EmbeddedFrame,
    ) -> Result<()> {
        frame.check(frame_types::FrameType0::FRAME_SIZE.bytes());
        let addr = frame.ptr().addr();
        let slot = get_user_image_frame_slot(self.bootinfo, &self.user_image_bounds, addr);
        self.set_orig_cslot(obj_id, slot);
        Ok(())
    }

    fn init_irqs(&mut self) -> Result<()> {
        debug!("Initializing IRQs");

        let irq_notifications = self
            .spec()
            .filter_objects::<&object::IRQ>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let arm_irq_notifications = self
            .spec()
            .filter_objects::<&object::ArmIRQ>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));

        for (obj_id, notification) in irq_notifications.chain(arm_irq_notifications) {
            let irq_handler = self.orig_local_cptr::<cap_type::IRQHandler>(obj_id);
            if let Some(logical_nfn_cap) = notification {
                let nfn = match logical_nfn_cap.badge {
                    0 => self.orig_local_cptr(logical_nfn_cap.object),
                    badge => {
                        let orig_cptr = self.orig_relative_cptr(logical_nfn_cap.object);
                        let slot = self.cslot_alloc_or_panic();
                        let cptr = cslot_relative_cptr(slot);
                        cptr.mint(&orig_cptr, CapRights::all(), badge)?;
                        cslot_local_cptr(slot)
                    }
                };
                irq_handler.irq_handler_set_notification(nfn)?;
            }
        }
        Ok(())
    }

    fn init_asids(&self) -> Result<()> {
        debug!("Initializing ASIDs");
        for (obj_id, _obj) in self
            .spec()
            .filter_objects_with::<&object::PageTable>(|obj| obj.is_root)
        {
            let vspace = self.orig_local_cptr::<cap_type::VSpace>(obj_id);
            BootInfo::init_thread_asid_pool().asid_pool_assign(vspace)?;
        }
        Ok(())
    }

    fn init_frames(&mut self) -> Result<()> {
        debug!("Initializing Frames");
        for (obj_id, obj) in self.spec().filter_objects::<&object::Frame<'a, D, M>>() {
            // TODO make more platform-agnostic
            if let Some(fill) = obj.init.as_fill() {
                let entries = &fill.entries;
                if !entries.is_empty() {
                    match obj.size_bits {
                        frame_types::FRAME_SIZE_0_BITS => {
                            let frame = self.orig_local_cptr::<frame_types::FrameType0>(obj_id);
                            self.fill_frame(frame, entries)?;
                        }
                        frame_types::FRAME_SIZE_1_BITS => {
                            let frame = self.orig_local_cptr::<frame_types::FrameType1>(obj_id);
                            self.fill_frame(frame, entries)?;
                        }
                        _ => {
                            panic!()
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn fill_frame<U: FrameType>(&self, frame: LocalCPtr<U>, fill: &[FillEntry<D>]) -> Result<()> {
        frame.frame_map(
            BootInfo::init_thread_vspace(),
            self.copy_addr::<U>(),
            CapRights::read_write(),
            arch::vm_attributes_from_whether_cached(false),
        )?;
        atomic::fence(Ordering::SeqCst); // lazy
        for entry in fill.iter() {
            let offset = entry.range.start;
            let length = entry.range.end - entry.range.start;
            assert!(entry.range.end <= U::FRAME_SIZE.bytes());
            let dst_frame = ptr::from_exposed_addr_mut::<u8>(self.copy_addr::<U>());
            let dst = unsafe { slice::from_raw_parts_mut(dst_frame.add(offset), length) };
            match &entry.content {
                FillEntryContent::Data(content_data) => {
                    content_data.copy_out(self.spec_with_sources.content_source, dst);
                }
                FillEntryContent::BootInfo(content_bootinfo) => {
                    for extra in self.bootinfo.extra() {
                        if extra.id == (&content_bootinfo.id).into() {
                            let n = dst.len().min(
                                extra
                                    .content_with_header()
                                    .len()
                                    .saturating_sub(content_bootinfo.offset),
                            );
                            if n > 0 {
                                dst[..n].copy_from_slice(
                                    &extra.content_with_header()
                                        [content_bootinfo.offset..(content_bootinfo.offset + n)],
                                );
                            }
                        }
                    }
                }
            }
        }
        atomic::fence(Ordering::SeqCst); // lazy
        frame.frame_unmap()?;
        Ok(())
    }

    fn init_vspaces(&mut self) -> Result<()> {
        debug!("Initializing VSpaces");
        self.init_vspaces_arch()
    }

    #[sel4::sel4_cfg(ARCH_AARCH64)]
    fn init_vspaces_arch(&mut self) -> Result<()> {
        // TODO
        // Add support for uncached non-device mappings.
        // See note about seL4_ARM_Page_CleanInvalidate_Data/seL4_ARM_Page_Unify_Instruction in upstream.

        for (obj_id, obj) in self
            .spec()
            .filter_objects_with::<&object::PageTable>(|obj| obj.is_root)
        {
            let vspace = self.orig_local_cptr::<cap_type::VSpace>(obj_id);
            for (i, cap) in obj.page_tables() {
                let pt_level_one = self.orig_local_cptr::<cap_type::PT>(cap.object);
                let vaddr = i << cap_type::PT::SPAN_BITS;
                pt_level_one.pt_map(vspace, vaddr, cap.vm_attributes())?;
                for (i, cap) in self
                    .spec()
                    .lookup_object::<&object::PageTable>(cap.object)?
                    .page_tables()
                {
                    let pt_level_two = self.orig_local_cptr::<cap_type::PT>(cap.object);
                    let vaddr = vaddr + (i << cap_type::PT::SPAN_BITS);
                    pt_level_two.pt_map(vspace, vaddr, cap.vm_attributes())?;
                    for (i, cap) in self
                        .spec()
                        .lookup_object::<&object::PageTable>(cap.object)?
                        .entries()
                    {
                        let vaddr = vaddr + (i << cap_type::PT::SPAN_BITS);
                        match cap {
                            PageTableEntry::Frame(cap) => {
                                let frame = self.orig_local_cptr::<cap_type::LargePage>(cap.object);
                                let rights = (&cap.rights).into();
                                self.copy(frame)?.frame_map(
                                    vspace,
                                    vaddr,
                                    rights,
                                    cap.vm_attributes(),
                                )?;
                            }
                            PageTableEntry::PageTable(cap) => {
                                let pt_level_three =
                                    self.orig_local_cptr::<cap_type::PT>(cap.object);
                                pt_level_three.pt_map(vspace, vaddr, cap.vm_attributes())?;
                                for (i, cap) in self
                                    .spec()
                                    .lookup_object::<&object::PageTable>(cap.object)?
                                    .frames()
                                {
                                    let frame =
                                        self.orig_local_cptr::<cap_type::SmallPage>(cap.object);
                                    let vaddr = vaddr + (i << FrameSize::Small.bits());
                                    let rights = (&cap.rights).into();
                                    self.copy(frame)?.frame_map(
                                        vspace,
                                        vaddr,
                                        rights,
                                        cap.vm_attributes(),
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[sel4::sel4_cfg(ARCH_X86_64)]
    fn init_vspaces_arch(&mut self) -> Result<()> {
        for (obj_id, obj) in self
            .spec()
            .filter_objects_with::<&object::PageTable>(|obj| obj.is_root)
        {
            let vspace = self.orig_local_cptr::<cap_type::VSpace>(obj_id);
            for (i, cap) in obj.page_tables() {
                let pdpt = self.orig_local_cptr::<cap_type::PDPT>(cap.object);
                let vaddr = i << cap_type::PDPT::SPAN_BITS;
                pdpt.pdpt_map(vspace, vaddr, cap.vm_attributes())?;
                for (i, cap) in self
                    .spec()
                    .lookup_object::<&object::PageTable>(cap.object)?
                    .page_tables()
                {
                    let page_directory =
                        self.orig_local_cptr::<cap_type::PageDirectory>(cap.object);
                    let vaddr = vaddr + (i << cap_type::PageDirectory::SPAN_BITS);
                    page_directory.page_directory_map(vspace, vaddr, cap.vm_attributes())?;
                    for (i, cap) in self
                        .spec()
                        .lookup_object::<&object::PageTable>(cap.object)?
                        .entries()
                    {
                        let vaddr = vaddr + (i << cap_type::PageTable::SPAN_BITS);
                        match cap {
                            PageTableEntry::Frame(cap) => {
                                let frame = self.orig_local_cptr::<cap_type::LargePage>(cap.object);
                                let rights = (&cap.rights).into();
                                self.copy(frame)?.frame_map(
                                    vspace,
                                    vaddr,
                                    rights,
                                    cap.vm_attributes(),
                                )?;
                            }
                            PageTableEntry::PageTable(cap) => {
                                let page_table =
                                    self.orig_local_cptr::<cap_type::PageTable>(cap.object);
                                page_table.page_table_map(vspace, vaddr, cap.vm_attributes())?;
                                for (i, cap) in self
                                    .spec()
                                    .lookup_object::<&object::PageTable>(cap.object)?
                                    .frames()
                                {
                                    let frame = self.orig_local_cptr::<cap_type::_4K>(cap.object);
                                    let vaddr = vaddr + (i << FrameSize::_4K.bits());
                                    let rights = (&cap.rights).into();
                                    self.copy(frame)?.frame_map(
                                        vspace,
                                        vaddr,
                                        rights,
                                        cap.vm_attributes(),
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[sel4::sel4_cfg(KERNEL_MCS)]
    fn init_sched_contexts(&self) -> Result<()> {
        debug!("Initializing scheduling contexts");
        for (obj_id, _obj) in self.spec().filter_objects::<&object::SchedContext>() {
            self.init_sched_context(obj_id, 0)?;
        }
        Ok(())
    }

    #[sel4::sel4_cfg(KERNEL_MCS)]
    fn init_sched_context(&self, obj_id: ObjectId, affinity: usize) -> Result<()> {
        let obj = self.spec().lookup_object::<&object::SchedContext>(obj_id)?;
        let sched_context = self.orig_local_cptr::<cap_type::SchedContext>(obj_id);
        self.bootinfo
            .sched_control(affinity)
            .sched_control_configure_flags(
                sched_context,
                obj.extra.budget,
                obj.extra.period,
                0,
                obj.extra.badge,
                0,
            )?;
        Ok(())
    }

    fn init_tcbs(&mut self) -> Result<()> {
        debug!("Initializing TCBs");

        for (obj_id, obj) in self.spec().filter_objects::<&object::TCB>() {
            let tcb = self.orig_local_cptr::<cap_type::TCB>(obj_id);

            if let Some(bound_notification) = obj.bound_notification() {
                let bound_notification =
                    self.orig_local_cptr::<cap_type::Notification>(bound_notification.object);
                tcb.tcb_bind_notification(bound_notification)?;
            }

            #[sel4::sel4_cfg(all(ARCH_AARCH64, ARM_HYPERVISOR_SUPPORT))]
            {
                if let Some(vcpu) = obj.vcpu() {
                    let vcpu = self.orig_local_cptr::<cap_type::VCPU>(vcpu.object);
                    vcpu.vcpu_set_tcb(tcb)?;
                }
            }

            {
                let cspace = self.orig_local_cptr(obj.cspace().object);
                let cspace_root_data = CNodeCapData::new(
                    obj.cspace().guard,
                    obj.cspace().guard_size.try_into().unwrap(),
                );
                let vspace = self.orig_local_cptr(obj.vspace().object);
                let ipc_buffer_addr = obj.extra.ipc_buffer_addr;
                let ipc_buffer_frame = self.orig_local_cptr(obj.ipc_buffer().object);

                let authority = BootInfo::init_thread_tcb();
                let max_prio = obj.extra.max_prio.try_into()?;
                let prio = obj.extra.prio.try_into()?;

                #[allow(unused_variables)]
                let affinity: usize = obj.extra.affinity.try_into()?;

                sel4::sel4_cfg_if! {
                    if #[cfg(KERNEL_MCS)] {
                        if let Some(sched_context_cap) = obj.sc() {
                            self.init_sched_context(sched_context_cap.object, affinity)?;
                        }

                        tcb.tcb_configure(
                            cspace,
                            cspace_root_data,
                            vspace,
                            ipc_buffer_addr,
                            ipc_buffer_frame,
                        )?;

                        let sc = match obj.sc() {
                            None => BootInfo::null().cast::<cap_type::SchedContext>(),
                            Some(cap) => self.orig_local_cptr::<cap_type::SchedContext>(cap.object),
                        };

                        let fault_ep = match obj.temp_fault_ep() {
                            None => BootInfo::null().cast::<cap_type::Endpoint>(),
                            Some(cap) => {
                                let orig = self.orig_local_cptr::<cap_type::Endpoint>(cap.object);
                                let badge = cap.badge;
                                let rights = (&cap.rights).into();
                                if badge == 0 || rights == CapRights::all() {
                                    orig
                                } else {
                                    let src = BootInfo::init_thread_cnode().relative(orig);
                                    let new = BootInfo::init_cspace_local_cptr::<cap_type::Endpoint>(self.cslot_alloc_or_panic());
                                    let dst = BootInfo::init_thread_cnode().relative(new);
                                    dst.mint(&src, rights, badge)?;
                                    new
                                }
                            },
                        };

                        let temp_fault_ep = match obj.temp_fault_ep() {
                            None => BootInfo::null().cast::<cap_type::Endpoint>(),
                            Some(cap) => {
                                assert_eq!(cap.badge, 0); // HACK
                                self.orig_local_cptr::<cap_type::Endpoint>(cap.object)
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
                        let fault_ep = CPtr::from_bits(obj.extra.master_fault_ep.unwrap());

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

                        #[sel4::sel4_cfg(not(MAX_NUM_NODES = "1"))]
                        {
                            tcb.tcb_set_affinity(affinity.try_into().unwrap())?;
                        }
                    }
                }
            }

            {
                let mut regs = UserContext::default();
                arch::init_user_context(&mut regs, &obj.extra);
                tcb.tcb_write_all_registers(false, &mut regs)?;
            }

            if let Some(name) = self.object_name(self.spec().name(obj_id)) {
                tcb.debug_name(name.as_bytes());
            }
        }
        Ok(())
    }

    fn init_cspaces(&self) -> Result<()> {
        debug!("Initializing CSpaces");

        for (obj_id, obj) in self.spec().filter_objects::<&object::CNode>() {
            let cnode = self.orig_local_cptr::<cap_type::CNode>(obj_id);
            for (i, cap) in obj.slots() {
                let badge = cap.badge();
                let rights = cap.rights().map(From::from).unwrap_or(CapRights::all());
                let src = BootInfo::init_thread_cnode()
                    .relative(self.orig_local_cptr::<cap_type::Unspecified>(cap.obj()));
                let dst = cnode.relative_bits_with_depth((*i).try_into().unwrap(), obj.size_bits);
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
        for (obj_id, obj) in self.spec().filter_objects::<&object::TCB>() {
            let tcb = self.orig_local_cptr::<cap_type::TCB>(obj_id);
            if obj.extra.resume {
                tcb.tcb_resume()?;
            }
        }
        Ok(())
    }

    //

    fn copy_addr<U: FrameType>(&self) -> usize {
        match U::FRAME_SIZE {
            frame_types::FrameType0::FRAME_SIZE => self.small_frame_copy_addr,
            frame_types::FrameType1::FRAME_SIZE => self.large_frame_copy_addr,
            _ => unimplemented!(),
        }
    }

    //

    fn copy<U: CapType>(&mut self, cap: LocalCPtr<U>) -> Result<LocalCPtr<U>> {
        let slot = self.cslot_alloc_or_panic();
        let src = BootInfo::init_thread_cnode().relative(cap);
        cslot_relative_cptr(slot).copy(&src, CapRights::all())?;
        Ok(cslot_local_cptr(slot))
    }

    //

    fn cslot_alloc_or_panic(&mut self) -> InitCSpaceSlot {
        self.cslot_allocator.alloc_or_panic()
    }

    fn set_orig_cslot(&mut self, obj_id: ObjectId, slot: InitCSpaceSlot) {
        self.buffers.per_obj_mut()[obj_id].orig_slot = Some(slot);
    }

    fn orig_cslot(&self, obj_id: ObjectId) -> InitCSpaceSlot {
        self.buffers.per_obj()[obj_id].orig_slot.unwrap()
    }

    fn alloc_orig_cslot(&mut self, obj_id: ObjectId) -> InitCSpaceSlot {
        let slot = self.cslot_alloc_or_panic();
        self.set_orig_cslot(obj_id, slot);
        slot
    }

    fn orig_local_cptr<U: CapType>(&self, obj_id: ObjectId) -> LocalCPtr<U> {
        cslot_local_cptr(self.orig_cslot(obj_id))
    }

    fn orig_relative_cptr(&self, obj_id: ObjectId) -> AbsoluteCPtr {
        cslot_relative_cptr(self.orig_cslot(obj_id))
    }

    fn ut_local_cptr(&self, ut_index: usize) -> Untyped {
        BootInfo::init_cspace_local_cptr(self.bootinfo.untyped().start + ut_index)
    }
}

fn cslot_local_cptr<T: CapType>(slot: InitCSpaceSlot) -> LocalCPtr<T> {
    BootInfo::init_cspace_local_cptr(slot)
}

fn cslot_cptr(slot: InitCSpaceSlot) -> CPtr {
    BootInfo::init_cspace_cptr(slot)
}

fn cslot_relative_cptr(slot: InitCSpaceSlot) -> AbsoluteCPtr {
    BootInfo::init_thread_cnode().relative(cslot_cptr(slot))
}

fn init_thread_cnode_relative_cptr() -> AbsoluteCPtr {
    BootInfo::init_thread_cnode().relative(BootInfo::init_thread_cnode())
}
