//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::Range;

use crate::{
    BytesContent, Content, DeflatedBytesContent, EmbeddedFrameIndex, Fill, FillEntry,
    FillEntryContent, FrameInit, NamedObject, Object, ObjectId, OrigCapSlots, Spec,
    SpecForInitializer, object,
};

impl<D> Spec<Fill<D>> {
    pub fn embed_fill(
        &self,
        granule_size_bits: u8,
        should_embed: impl FnMut(&Fill<D>) -> bool,
        mut f: impl FnMut(&D, &mut [u8]) -> bool,
    ) -> (SpecForInitializer, Vec<Vec<u8>>) {
        self.embed_fill_fallible(granule_size_bits, should_embed, |x1, x2| Ok(f(x1, x2)))
            .unwrap_or_else(|absurdity: Infallible| match absurdity {})
    }

    pub fn embed_fill_fallible<E>(
        &self,
        granule_size_bits: u8,
        mut should_embed: impl FnMut(&Fill<D>) -> bool,
        mut f: impl FnMut(&D, &mut [u8]) -> Result<bool, E>,
    ) -> Result<(SpecForInitializer, Vec<Vec<u8>>), E> {
        let granule_size = 1 << granule_size_bits;
        let mut frame_data = vec![];
        let spec = self.traverse_frame_init_fallible(|frame, is_root| {
            Ok(
                if should_embed(&frame.init) && can_embed(granule_size_bits, frame, is_root) {
                    FrameInit::Embedded({
                        let mut frame_buf = vec![0; granule_size];
                        for entry in frame.init.entries.iter() {
                            f(
                                entry.content.as_data().unwrap(),
                                &mut frame_buf[try_into_usize_range(&entry.range).unwrap()],
                            )?;
                        }
                        let ix = frame_data.len();
                        frame_data.push(frame_buf);
                        EmbeddedFrameIndex {
                            index: ix.try_into().unwrap(),
                        }
                    })
                } else {
                    FrameInit::Fill({
                        frame.init.traverse_fallible(|range, data| {
                            let length = (range.end - range.start).try_into().unwrap();
                            let mut buf = vec![0; length];
                            let deflate = f(data, &mut buf)?;
                            Ok(if deflate {
                                Content::DeflatedBytes(DeflatedBytesContent::pack(&buf))
                            } else {
                                Content::Bytes(BytesContent::pack(&buf))
                            })
                        })?
                    })
                },
            )
        })?;
        Ok((spec, frame_data))
    }
}

impl<D> Spec<D> {
    fn traverse_frame_init_fallible<D1, E>(
        &self,
        mut f: impl FnMut(&object::Frame<D>, bool) -> Result<D1, E>,
    ) -> Result<Spec<D1>, E> {
        Ok(Spec {
            objects: self
                .objects
                .iter()
                .enumerate()
                .map(|(obj_id, named_obj)| {
                    Ok(NamedObject {
                        name: named_obj.name.clone(),
                        object: match &named_obj.object {
                            Object::Untyped(obj) => Object::Untyped(obj.clone()),
                            Object::Endpoint => Object::Endpoint,
                            Object::Notification => Object::Notification,
                            Object::CNode(obj) => Object::CNode(obj.clone()),
                            Object::Tcb(obj) => Object::Tcb(obj.clone()),
                            Object::Irq(obj) => Object::Irq(obj.clone()),
                            Object::VCpu => Object::VCpu,
                            Object::Frame(obj) => Object::Frame(object::Frame {
                                size_bits: obj.size_bits,
                                paddr: obj.paddr,
                                init: {
                                    let is_root = ObjectId::into_usize_range(&self.root_objects)
                                        .contains(&obj_id);
                                    f(obj, is_root)?
                                },
                            }),
                            Object::PageTable(obj) => Object::PageTable(obj.clone()),
                            Object::AsidPool(obj) => Object::AsidPool(obj.clone()),
                            Object::ArmIrq(obj) => Object::ArmIrq(obj.clone()),
                            Object::IrqMsi(obj) => Object::IrqMsi(obj.clone()),
                            Object::IrqIOApic(obj) => Object::IrqIOApic(obj.clone()),
                            Object::RiscvIrq(obj) => Object::RiscvIrq(obj.clone()),
                            Object::IOPorts(obj) => Object::IOPorts(obj.clone()),
                            Object::SchedContext(obj) => Object::SchedContext(obj.clone()),
                            Object::Reply => Object::Reply,
                            Object::ArmSmc => Object::ArmSmc,
                        },
                    })
                })
                .collect::<Result<_, E>>()?,
            irqs: self.irqs.clone(),
            asid_slots: self.asid_slots.clone(),
            root_objects: self.root_objects.clone(),
            untyped_covers: self.untyped_covers.clone(),
            cached_orig_cap_slots: self.cached_orig_cap_slots.clone(),
        })
    }
}

impl<D> Fill<D> {
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn depends_on_bootinfo(&self) -> bool {
        self.entries.iter().any(|entry| entry.content.is_bootinfo())
    }

    fn traverse_fallible<D1, E>(
        &self,
        mut f: impl FnMut(&Range<u64>, &D) -> Result<D1, E>,
    ) -> Result<Fill<D1>, E> {
        Ok(Fill {
            entries: self
                .entries
                .iter()
                .map(|entry| {
                    Ok(FillEntry {
                        range: entry.range.clone(),
                        content: match &entry.content {
                            FillEntryContent::BootInfo(content_bootinfo) => {
                                FillEntryContent::BootInfo(*content_bootinfo)
                            }
                            FillEntryContent::Data(content_data) => {
                                FillEntryContent::Data(f(&entry.range, content_data)?)
                            }
                        },
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }
}

fn can_embed<D>(granule_size_bits: u8, frame: &object::Frame<Fill<D>>, is_root: bool) -> bool {
    is_root
        && frame.paddr.is_none()
        && frame.size_bits == granule_size_bits
        && !frame.init.is_empty()
        && !frame.init.depends_on_bootinfo()
}

fn try_into_usize_range<T: TryInto<usize> + Copy>(
    range: &Range<T>,
) -> Result<Range<usize>, <T as TryInto<usize>>::Error> {
    Ok(range.start.try_into()?..range.end.try_into()?)
}

impl SpecForInitializer {
    pub fn cache_orig_cap_slots(&mut self) {
        let mut num_occupied = 0;
        let offsets_by_object = self
            .objects
            .iter()
            .map(|named_obj| {
                if named_obj.object.needs_orig_cap_slot() {
                    let offset = num_occupied;
                    num_occupied += 1;
                    Some(offset)
                } else {
                    None
                }
            })
            .collect();
        self.cached_orig_cap_slots = Some(OrigCapSlots {
            num_occupied,
            offsets_by_object,
        })
    }
}

impl Object<FrameInit> {
    fn needs_orig_cap_slot(&self) -> bool {
        match self {
            Self::ArmSmc => false,
            Self::Frame(frame) => !frame.init.is_embedded(),
            _ => true,
        }
    }
}
