//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;

use crate::{
    object, Fill, FillEntry, FillEntryContent, FrameInit, NamedObject, NeverEmbedded, Object, Spec,
};

impl<'a, N, D, M> Spec<'a, N, D, M> {
    pub(crate) fn traverse<N1, D1, M1, E>(
        &self,
        mut f: impl FnMut(&NamedObject<'a, N, D, M>) -> Result<N1, E>,
        mut g: impl FnMut(&object::Frame<'a, D, M>, bool) -> Result<FrameInit<'a, D1, M1>, E>,
    ) -> Result<Spec<'a, N1, D1, M1>, E> {
        Ok(Spec {
            objects: self
                .objects
                .iter()
                .enumerate()
                .map(|(obj_id, named_obj)| {
                    Ok(NamedObject {
                        name: f(named_obj)?,
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
                                init: g(obj, self.root_objects.contains(&obj_id))?,
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
        })
    }
}

impl<'a, N, D: Clone, M: Clone> Spec<'a, N, D, M> {
    pub fn traverse_names_with_context_fallible<N1, E>(
        &self,
        f: impl FnMut(&NamedObject<'a, N, D, M>) -> Result<N1, E>,
    ) -> Result<Spec<'a, N1, D, M>, E> {
        self.traverse(f, |frame, _is_root| Ok(frame.init.clone()))
    }

    pub fn traverse_names_with_context<N1>(
        &self,
        mut f: impl FnMut(&NamedObject<'a, N, D, M>) -> N1,
    ) -> Spec<'a, N1, D, M> {
        unwrap_infallible(self.traverse_names_with_context_fallible(|x| Ok(f(x))))
    }

    pub fn traverse_names_fallible<N1, E>(
        &self,
        mut f: impl FnMut(&N) -> Result<N1, E>,
    ) -> Result<Spec<'a, N1, D, M>, E> {
        self.traverse_names_with_context_fallible(|named_object| f(&named_object.name))
    }

    pub fn traverse_names<N1>(&self, mut f: impl FnMut(&N) -> N1) -> Spec<'a, N1, D, M> {
        unwrap_infallible(self.traverse_names_fallible(|x| Ok(f(x))))
    }
}

impl<'a, N: Clone, D, M> Spec<'a, N, D, M> {
    pub(crate) fn traverse_frame_init<D1, M1, E>(
        &self,
        f: impl FnMut(&object::Frame<'a, D, M>, bool) -> Result<FrameInit<'a, D1, M1>, E>,
    ) -> Result<Spec<'a, N, D1, M1>, E> {
        self.traverse(|named_object| Ok(named_object.name.clone()), f)
    }
}

impl<'a, N: Clone, D, M: Clone> Spec<'a, N, D, M> {
    pub fn traverse_data_with_context_fallible<D1, E>(
        &self,
        mut f: impl FnMut(usize, &D) -> Result<D1, E>,
    ) -> Result<Spec<'a, N, D1, M>, E> {
        self.traverse_frame_init(|frame, _is_root| {
            Ok(match &frame.init {
                FrameInit::Fill(fill) => FrameInit::Fill(Fill {
                    entries: fill
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
                                        FillEntryContent::Data(f(entry.range.len(), content_data)?)
                                    }
                                },
                            })
                        })
                        .collect::<Result<_, E>>()?,
                }),
                FrameInit::Embedded(embedded) => FrameInit::Embedded(embedded.clone()),
            })
        })
    }

    pub fn traverse_data_with_context<D1>(
        &self,
        mut f: impl FnMut(usize, &D) -> D1,
    ) -> Spec<'a, N, D1, M> {
        unwrap_infallible(self.traverse_data_with_context_fallible(|x1, x2| Ok(f(x1, x2))))
    }

    pub fn traverse_data_fallible<D1, E>(
        &self,
        mut f: impl FnMut(&D) -> Result<D1, E>,
    ) -> Result<Spec<'a, N, D1, M>, E> {
        self.traverse_data_with_context_fallible(|_length, data| f(data))
    }

    pub fn traverse_data<D1>(&self, mut f: impl FnMut(&D) -> D1) -> Spec<'a, N, D1, M> {
        unwrap_infallible(self.traverse_data_fallible(|x| Ok(f(x))))
    }
}

impl<'a, N: Clone, D: Clone, M> Spec<'a, N, D, M> {
    pub fn traverse_embedded_frames_fallible<M1, E>(
        &self,
        mut f: impl FnMut(&M) -> Result<M1, E>,
    ) -> Result<Spec<'a, N, D, M1>, E> {
        self.traverse_frame_init(|frame, _is_root| {
            Ok(match &frame.init {
                FrameInit::Fill(fill) => FrameInit::Fill(fill.clone()),
                FrameInit::Embedded(embedded) => FrameInit::Embedded(f(embedded)?),
            })
        })
    }

    pub fn traverse_embedded_frames<M1>(&self, mut f: impl FnMut(&M) -> M1) -> Spec<'a, N, D, M1> {
        unwrap_infallible(self.traverse_embedded_frames_fallible(|x| Ok(f(x))))
    }
}

impl<'a, N: Clone, D: Clone> Spec<'a, N, D, NeverEmbedded> {
    pub fn split_embedded_frames(
        &self,
        embed_frames: bool,
        granule_size_bits: usize,
    ) -> Spec<'a, N, D, Fill<'a, D>> {
        unwrap_infallible(
            self.traverse_frame_init::<_, _, Infallible>(|frame, is_root| {
                let fill = frame.init.as_fill_infallible();
                Ok(
                    if embed_frames && frame.can_embed(granule_size_bits, is_root) {
                        FrameInit::Embedded(fill.clone())
                    } else {
                        FrameInit::Fill(fill.clone())
                    },
                )
            }),
        )
    }
}

fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    result.unwrap_or_else(|absurdity| match absurdity {})
}
