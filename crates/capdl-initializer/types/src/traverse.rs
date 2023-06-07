use crate::{object, Fill, FillEntry, FillEntryContent, FrameInit, NamedObject, Object, Spec};

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
                            Object::TCB(obj) => Object::TCB(obj.clone()),
                            Object::IRQ(obj) => Object::IRQ(obj.clone()),
                            Object::VCPU => Object::VCPU,
                            Object::Frame(obj) => Object::Frame(object::Frame {
                                size_bits: obj.size_bits,
                                paddr: obj.paddr,
                                init: g(&obj, self.root_objects.contains(&obj_id))?,
                            }),
                            Object::PageTable(obj) => Object::PageTable(obj.clone()),
                            Object::ASIDPool(obj) => Object::ASIDPool(obj.clone()),
                            Object::ArmIRQ(obj) => Object::ArmIRQ(obj.clone()),
                            Object::SchedContext(obj) => Object::SchedContext(obj.clone()),
                            Object::Reply => Object::Reply,
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
    pub fn traverse_names_with_context<N1, E>(
        &self,
        f: impl FnMut(&NamedObject<'a, N, D, M>) -> Result<N1, E>,
    ) -> Result<Spec<'a, N1, D, M>, E> {
        self.traverse(f, |frame, _is_root| Ok(frame.init.clone()))
    }

    pub fn traverse_names<N1, E>(
        &self,
        mut f: impl FnMut(&N) -> Result<N1, E>,
    ) -> Result<Spec<'a, N1, D, M>, E> {
        self.traverse_names_with_context(|named_object| f(&named_object.name))
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
    pub fn traverse_data_with_context<D1, E>(
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

    pub fn traverse_data<D1, E>(
        &self,
        mut f: impl FnMut(&D) -> Result<D1, E>,
    ) -> Result<Spec<'a, N, D1, M>, E> {
        self.traverse_data_with_context(|_length, data| f(data))
    }
}

impl<'a, N: Clone, D: Clone, M> Spec<'a, N, D, M> {
    pub fn traverse_embedded_frames<M1, E>(
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
}

impl<'a, N: Clone, D: Clone> Spec<'a, N, D, !> {
    pub fn split_embedded_frames(
        &self,
        embed_frames: bool,
        granule_size_bits: usize,
    ) -> Spec<'a, N, D, Fill<'a, D>> {
        self.traverse_frame_init::<_, _, !>(|frame, is_root| {
            let fill = frame.init.as_fill_infallible();
            Ok(
                if embed_frames && frame.can_embed(granule_size_bits, is_root) {
                    FrameInit::Embedded(fill.clone())
                } else {
                    FrameInit::Fill(fill.clone())
                },
            )
        })
        .into_ok()
    }
}
