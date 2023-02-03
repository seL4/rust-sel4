#![no_std]
#![feature(array_try_from_fn)]
#![feature(core_intrinsics)]
#![feature(ptr_to_from_bits)]
#![feature(strict_provenance)]
#![feature(proc_macro_hygiene)]
#![feature(int_roundings)]
#![feature(never_type)]
#![feature(const_trait_impl)]
#![allow(unused_variables)]

use core::array;
use core::borrow::BorrowMut;
use core::ops::Range;
use core::ptr;
use core::result;
use core::slice;
use core::sync::atomic::{self, Ordering};

#[allow(unused_imports)]
use log::{debug, info, trace};

use capdl_types::*;
use sel4::{
    cap_type, AbsoluteCPtr, BootInfo, CNodeCapData, CPtr, CapRights, CapType, FrameSize, FrameType,
    InitCSpaceSlot, LocalCPtr, ObjectBlueprint, TranslationTableType, Untyped, UserContext,
    VMAttributes,
};

mod buffers;
mod cslot_allocator;
mod error;
mod hold_slots;
mod memory;
mod utils;

pub use buffers::{LoaderBuffers, PerObjectBuffer};
use cslot_allocator::{CSlotAllocator, CSlotAllocatorError};
pub use error::CapDLLoaderError;
use hold_slots::HoldSlots;
use memory::init_copy_addrs;
use utils::{round_down, round_up};

// TODO see seL4_ARM_Page_CleanInvalidate_Data/seL4_ARM_Page_Unify_Instruction in upstream

type Fill<'a, T, C> = ContainerType<'a, T, FillEntry<C>>;
type CapTable<'a, T> = ContainerType<'a, T, CapTableEntry>;

type Result<T> = result::Result<T, CapDLLoaderError>;

pub fn load<
    'a,
    'b,
    T: Container<'b>,
    PO: BorrowMut<[PerObjectBuffer]>,
    S: ObjectName,
    C: AvailableFillEntryContentVia,
>(
    spec: &'b ConcreteSpec<'b, T, C, S>,
    via: &'b C::Via,
    bootinfo: &'a BootInfo,
    buffers: &'a mut LoaderBuffers<PO>,
    own_footprint: Range<usize>,
) -> Result<!> {
    info!("Starting CapDL Loader");

    let (small_frame_copy_addr, large_frame_copy_addr) = init_copy_addrs(bootinfo, &own_footprint)?;

    let mut cslot_allocator = CSlotAllocator::new(bootinfo.empty());

    Loader {
        spec,
        via,
        bootinfo,
        small_frame_copy_addr,
        large_frame_copy_addr,
        cslot_allocator: &mut cslot_allocator,
        buffers,
    }
    .load()
}

struct Loader<
    'a,
    'b,
    T: Container<'b>,
    PO: BorrowMut<[PerObjectBuffer]>,
    S,
    C: AvailableFillEntryContentVia,
> {
    spec: &'b ConcreteSpec<'b, T, C, S>,
    via: &'b C::Via,
    bootinfo: &'a BootInfo,
    small_frame_copy_addr: usize,
    large_frame_copy_addr: usize,
    cslot_allocator: &'a mut CSlotAllocator,
    buffers: &'a mut LoaderBuffers<PO>,
}

impl<
        'a,
        'b,
        T: Container<'b>,
        PO: BorrowMut<[PerObjectBuffer]>,
        S: ObjectName,
        C: AvailableFillEntryContentVia,
    > Loader<'a, 'b, T, PO, S, C>
{
    pub fn load(&mut self) -> Result<!> {
        self.create_objects()?;

        self.init_irqs()?;
        self.init_asids()?;
        self.init_frames()?;
        self.init_vspaces()?;
        self.init_tcbs()?;
        self.init_cspaces()?;

        self.start_threads()?;

        info!("CapDL Loader done, suspending");

        BootInfo::init_thread_tcb().tcb_suspend()?;

        unreachable!()
    }

    fn create_objects(&mut self) -> Result<()> {
        // This algorithm differs from that found in the upstream C CapDL
        // Loader. In particular, this one is implemented with objects
        // specifying non-device paddrs in mind.

        debug!("Creating objects");

        // Allocate CSlots
        {
            for obj_id in 0..self.spec.num_objects() {
                let slot = self.cslot_alloc_or_panic();
                self.set_orig_cslot(obj_id, slot);
            }
        }

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

        // Index objects

        let first_obj_without_paddr = self
            .spec
            .objects
            .as_slice()
            .partition_point(|named_obj| named_obj.object.paddr().is_some());
        let num_objs_with_paddr = first_obj_without_paddr;

        let mut by_size_start: [usize; sel4::WORD_SIZE] = array::from_fn(|_| 0);
        let mut by_size_end: [usize; sel4::WORD_SIZE] = array::from_fn(|_| 0);
        {
            for obj_id in first_obj_without_paddr..self.spec.num_objects() {
                let obj = &self.spec.object(obj_id);
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
        let mut hold_slots = HoldSlots::new(&mut self.cslot_allocator, cslot_relative_cptr)?;

        // Create objects

        let mut next_obj_with_paddr = 0;
        for i_ut in uts_by_paddr.iter() {
            let ut = &uts[*i_ut];
            let ut_size_bits = ut.size_bits();
            let ut_size_bytes = 1 << ut_size_bits;
            let ut_paddr_start = ut.paddr();
            let ut_paddr_end = ut_paddr_start + ut_size_bytes;
            let mut cur_paddr = ut_paddr_start;
            loop {
                let target = if next_obj_with_paddr < num_objs_with_paddr {
                    ut_paddr_end.min(self.spec.object(next_obj_with_paddr).paddr().unwrap())
                } else {
                    ut_paddr_end
                };
                while cur_paddr < target {
                    let max_size_bits = usize::try_from(cur_paddr.trailing_zeros())
                        .unwrap()
                        .min((target - cur_paddr).trailing_zeros().try_into().unwrap());
                    let mut created = false;
                    if !ut.is_device() {
                        for size_bits in (0..=max_size_bits).rev() {
                            let obj_id = &mut by_size_start[size_bits];
                            if *obj_id < by_size_end[size_bits] {
                                let named_obj = &self.spec.named_object(*obj_id);
                                let blueprint = named_obj.object.blueprint().unwrap();
                                assert_eq!(blueprint.physical_size_bits(), size_bits);
                                trace!(
                                    "Creating kernel object: paddr=0x{:x}, size_bits={} name='{}'",
                                    cur_paddr,
                                    blueprint.physical_size_bits(),
                                    named_obj.name
                                );
                                self.ut_local_cptr(*i_ut).untyped_retype(
                                    &blueprint,
                                    &init_thread_cnode_relative_cptr(),
                                    self.orig_cslot(*obj_id),
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
                        if next_obj_with_paddr < num_objs_with_paddr
                            && cur_paddr < self.spec.object(next_obj_with_paddr).paddr().unwrap()
                        {
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
                            cur_paddr = ut_paddr_end;
                        }
                    }
                }
                if next_obj_with_paddr < num_objs_with_paddr
                    && cur_paddr == self.spec.object(next_obj_with_paddr).paddr().unwrap()
                    && cur_paddr < ut_paddr_end
                {
                    let obj_id = next_obj_with_paddr;
                    let named_obj = &self.spec.named_object(obj_id);
                    let blueprint = named_obj.object.blueprint().unwrap();
                    trace!(
                        "Creating device object: paddr=0x{:x}, size_bits={} name='{}'",
                        cur_paddr,
                        blueprint.physical_size_bits(),
                        named_obj.name
                    );
                    self.ut_local_cptr(*i_ut).untyped_retype(
                        &blueprint,
                        &init_thread_cnode_relative_cptr(),
                        self.orig_cslot(obj_id),
                        1,
                    )?;
                    cur_paddr += 1 << blueprint.physical_size_bits();
                    next_obj_with_paddr += 1;
                } else {
                    break;
                }
            }
        }

        // Ensure that we've created every object
        for bits in 0..sel4::WORD_SIZE {
            assert_eq!(by_size_start[bits], by_size_end[bits]);
        }

        // Actually make the ASID pools. With the help of parse-capDL, we do
        // this in order of obj.asid_high, for verification reasons (see
        // upstream C CapDL loader).
        {
            for obj_id in self.spec.asid_slots.as_slice().iter() {
                let ut = self.orig_local_cptr(*obj_id);
                let slot = self.cslot_alloc_or_panic();
                BootInfo::asid_control().asid_control_make_pool(ut, &cslot_relative_cptr(slot))?;
                self.set_orig_cslot(*obj_id, slot);
            }
        }

        // Create IRQHandler caps
        {
            for IRQEntry { irq, handler } in self.spec.irqs.as_slice().iter() {
                let slot = self.cslot_alloc_or_panic();
                match self.spec.object(*handler) {
                    Object::ArmIRQ(obj) => {
                        sel4::sel4_cfg_if! {
                            if #[cfg(MAX_NUM_NODES = "1")] {
                                BootInfo::irq_control().irq_control_get_trigger(
                                    *irq,
                                    obj.trigger,
                                    &cslot_relative_cptr(slot),
                                )?;
                            } else {
                                BootInfo::irq_control().irq_control_get_trigger_core(
                                    *irq,
                                    obj.trigger,
                                    obj.target,
                                    &cslot_relative_cptr(slot),
                                )?;
                            }
                        }
                    }
                    Object::IRQ(_) => {
                        BootInfo::irq_control()
                            .irq_control_get(*irq, &cslot_relative_cptr(slot))?;
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

    fn init_irqs(&mut self) -> Result<()> {
        debug!("Initializing IRQs");

        let irq_notifications = self
            .spec
            .filter_objects::<&object::IRQ<CapTable<T>>>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let arm_irq_notifications = self
            .spec
            .filter_objects::<&object::ArmIRQ<CapTable<T>>>()
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
        for (obj_id, _obj) in self.spec.filter_objects::<&object::PGD<CapTable<T>>>() {
            let pgd = self.orig_local_cptr::<cap_type::PGD>(obj_id);
            BootInfo::init_thread_asid_pool().asid_pool_assign(pgd)?;
        }
        Ok(())
    }

    fn init_frames(&mut self) -> Result<()> {
        debug!("Initializing Frames");
        for (obj_id, obj) in self.spec.filter_objects::<&object::SmallPage<Fill<T, C>>>() {
            let frame = self.orig_local_cptr::<cap_type::SmallPage>(obj_id);
            self.fill_frame(frame, &obj.fill)?;
        }
        for (obj_id, obj) in self.spec.filter_objects::<&object::LargePage<Fill<T, C>>>() {
            let frame = self.orig_local_cptr::<cap_type::LargePage>(obj_id);
            self.fill_frame(frame, &obj.fill)?;
        }
        Ok(())
    }

    pub fn fill_frame<U: FrameType>(
        &self,
        frame: LocalCPtr<U>,
        fill: &Fill<'b, T, C>,
    ) -> Result<()> {
        frame.frame_map(
            BootInfo::init_thread_vspace(),
            self.copy_addr::<U>(),
            CapRights::read_write(),
            VMAttributes::default() & !VMAttributes::PAGE_CACHEABLE,
        )?;
        atomic::fence(Ordering::SeqCst);
        // atomic::compiler_fence(Ordering::SeqCst);
        for entry in fill.as_slice() {
            let offset = entry.range.start;
            let length = entry.range.end - entry.range.start;
            assert!(entry.range.end <= U::FRAME_SIZE.bytes());
            let dst_frame = ptr::from_exposed_addr_mut::<u8>(self.copy_addr::<U>());
            let dst = unsafe { slice::from_raw_parts_mut(dst_frame.add(offset), length) };
            match &entry.content {
                FillEntryContent::Data(content_data) => {
                    content_data.copy_out_via(&self.via, dst);
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
        atomic::fence(Ordering::SeqCst);
        // atomic::compiler_fence(Ordering::SeqCst);
        frame.frame_unmap()?;
        Ok(())
    }

    fn init_vspaces(&mut self) -> Result<()> {
        debug!("Initializing VSpaces");

        for (obj_id, obj) in self.spec.filter_objects::<&object::PGD<CapTable<T>>>() {
            let pgd = self.orig_local_cptr::<cap_type::PGD>(obj_id);
            for (i, cap) in obj.entries() {
                let pud = self.orig_local_cptr::<cap_type::PUD>(cap.object);
                let vaddr = i << cap_type::PUD::SPAN_BITS;
                pud.translation_table_map(pgd, vaddr, cap.vm_attributes())?;
                for (i, cap) in self
                    .spec
                    .lookup_object::<&object::PUD<_>>(cap.object)?
                    .entries()
                {
                    let pd = self.orig_local_cptr::<cap_type::PD>(cap.object);
                    let vaddr = vaddr + (i << cap_type::PD::SPAN_BITS);
                    pd.translation_table_map(pgd, vaddr, cap.vm_attributes())?;
                    for (i, cap) in self
                        .spec
                        .lookup_object::<&object::PD<_>>(cap.object)?
                        .entries()
                    {
                        let vaddr = vaddr + (i << cap_type::PT::SPAN_BITS);
                        match cap {
                            PDEntry::LargePage(cap) => {
                                let frame = self.orig_local_cptr::<cap_type::LargePage>(cap.object);
                                let rights = (&cap.rights).into();
                                self.copy(frame)?.frame_map(
                                    pgd,
                                    vaddr,
                                    rights,
                                    cap.vm_attributes(),
                                )?;
                            }
                            PDEntry::PT(cap) => {
                                let pt = self.orig_local_cptr::<cap_type::PT>(cap.object);
                                pt.translation_table_map(pgd, vaddr, cap.vm_attributes())?;
                                for (i, cap) in self
                                    .spec
                                    .lookup_object::<&object::PT<_>>(cap.object)?
                                    .entries()
                                {
                                    let frame =
                                        self.orig_local_cptr::<cap_type::SmallPage>(cap.object);
                                    let vaddr = vaddr + (i << FrameSize::Small.bits());
                                    let rights = (&cap.rights).into();
                                    self.copy(frame)?.frame_map(
                                        pgd,
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

    fn init_tcbs(&self) -> Result<()> {
        debug!("Initializing TCBs");

        for (obj_id, obj) in self.spec.filter_objects::<&object::TCB<CapTable<T>>>() {
            let tcb = self.orig_local_cptr::<cap_type::TCB>(obj_id);

            if let Some(bound_notification) = obj.bound_notification() {
                let bound_notification =
                    self.orig_local_cptr::<cap_type::Notification>(bound_notification.object);
                tcb.tcb_bind_notification(bound_notification)?;
            }

            if let Some(vcpu) = obj.vcpu() {
                let vcpu = self.orig_local_cptr::<cap_type::VCPU>(vcpu.object);
                vcpu.vcpu_set_tcb(tcb)?;
            }

            {
                let fault_ep = CPtr::from_bits(obj.fault_ep);
                let cspace = self.orig_local_cptr(obj.cspace().object);
                let cspace_root_data = CNodeCapData::new(
                    obj.cspace().guard,
                    obj.cspace().guard_size.try_into().unwrap(),
                );
                let vspace = self.orig_local_cptr(obj.vspace().object);
                let ipc_buffer_addr = obj.extra_info.ipc_buffer_addr;
                let ipc_buffer_frame = self.orig_local_cptr(obj.ipc_buffer().object);

                tcb.tcb_configure(
                    fault_ep,
                    cspace,
                    cspace_root_data,
                    vspace,
                    ipc_buffer_addr,
                    ipc_buffer_frame,
                )?;
            }

            tcb.tcb_set_sched_params(
                BootInfo::init_thread_tcb(),
                obj.extra_info.max_prio.try_into()?,
                obj.extra_info.prio.try_into()?,
            )?;

            #[sel4::sel4_cfg(not(MAX_NUM_NODES = "1"))]
            tcb.tcb_set_affinity(obj.extra_info.affinity)?;

            {
                let mut regs = UserContext::default();
                *regs.pc_mut() = obj.extra_info.ip;
                *regs.sp_mut() = obj.extra_info.sp;
                *regs.spsr_mut() = obj.extra_info.spsr;
                for (i, value) in obj.init_args.iter().enumerate() {
                    if let Some(value) = value {
                        *regs.gpr_mut(i.try_into()?) = *value;
                    }
                }
                tcb.tcb_write_all_registers(false, &mut regs)?;
            }

            {
                if let Some(name) = self.spec.name(obj_id).object_name() {
                    tcb.debug_name(name);
                }
            }
        }
        Ok(())
    }

    fn init_cspaces(&self) -> Result<()> {
        debug!("Initializing CSpaces");

        for (obj_id, obj) in self.spec.filter_objects::<&object::CNode<CapTable<T>>>() {
            let cnode = self.orig_local_cptr::<cap_type::CNode>(obj_id);
            for (i, cap) in obj.slots() {
                // TODO
                // parse-capDL uses badge=0 to mean no badge. Is that good
                // enough, or do we ever need to actually use the badge value '0'?
                let mut badge = 0;
                let mut rights = CapRights::all();
                match cap {
                    Cap::Endpoint(cap) => {
                        badge = cap.badge;
                        rights = (&cap.rights).into();
                    }
                    Cap::Notification(cap) => {
                        badge = cap.badge;
                        rights = (&cap.rights).into();
                    }
                    Cap::CNode(cap) => {
                        badge = CNodeCapData::new(cap.guard, cap.guard_size.try_into().unwrap())
                            .into_word();
                    }
                    Cap::SmallPage(cap) => {
                        rights = (&cap.rights).into();
                    }
                    Cap::LargePage(cap) => {
                        rights = (&cap.rights).into();
                    }
                    _ => {}
                };
                let src = BootInfo::init_thread_cnode()
                    .relative(self.orig_local_cptr::<cap_type::Unspecified>(cap.obj()));
                let dst = cnode.relative_bits_with_depth((*i).try_into().unwrap(), obj.size_bits);
                match badge {
                    0 => dst.copy(&src, rights),
                    _ => dst.mint(&src, rights, badge),
                }?;
            }
        }
        Ok(())
    }

    fn start_threads(&self) -> Result<()> {
        debug!("Starting threads");
        for (obj_id, obj) in self.spec.filter_objects::<&object::TCB<CapTable<T>>>() {
            let tcb = self.orig_local_cptr::<cap_type::TCB>(obj_id);
            if obj.extra_info.resume {
                tcb.tcb_resume()?;
            }
        }
        Ok(())
    }

    ///

    fn copy_addr<U: FrameType>(&self) -> usize {
        match U::FRAME_SIZE {
            FrameSize::Small => self.small_frame_copy_addr,
            FrameSize::Large => self.large_frame_copy_addr,
            _ => unimplemented!(),
        }
    }

    ///

    fn copy<U: CapType>(&mut self, cap: LocalCPtr<U>) -> Result<LocalCPtr<U>> {
        let slot = self.cslot_alloc_or_panic();
        let src = BootInfo::init_thread_cnode().relative(cap);
        cslot_relative_cptr(slot).copy(&src, CapRights::all())?;
        Ok(cslot_local_cptr(slot))
    }

    ///

    fn cslot_alloc_or_panic(&mut self) -> InitCSpaceSlot {
        self.cslot_allocator.alloc_or_panic()
    }

    fn set_orig_cslot(&mut self, obj_id: ObjectId, slot: InitCSpaceSlot) {
        self.buffers.per_obj_mut()[obj_id].orig_slot = Some(slot);
    }

    fn orig_cslot(&self, obj_id: ObjectId) -> InitCSpaceSlot {
        self.buffers.per_obj()[obj_id].orig_slot.unwrap()
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
