use alloc::boxed::Box;

use crate::{object, FillEntry, FillEntryContent, Indirect, NamedObject, Object, Spec};

impl<'a, N, F> Spec<'a, N, F> {
    pub fn traverse<N1, F1, E>(
        &self,
        mut f: impl FnMut(&Object<'a, F>, &N) -> Result<N1, E>,
        mut g: impl FnMut(usize, &F) -> Result<F1, E>,
    ) -> Result<Spec<'a, N1, F1>, E> {
        Ok(Spec {
            objects: self
                .objects
                .traverse(|named_object| named_object.traverse(&mut f, &mut g))?,
            irqs: self.irqs.clone(),
            asid_slots: self.asid_slots.clone(),
        })
    }
}

impl<'a, N, F> NamedObject<'a, N, F> {
    pub fn traverse<N1, F1, E>(
        &self,
        f: impl FnOnce(&Object<'a, F>, &N) -> Result<N1, E>,
        g: impl FnMut(usize, &F) -> Result<F1, E>,
    ) -> Result<NamedObject<'a, N1, F1>, E> {
        Ok(NamedObject {
            name: f(&self.object, &self.name)?,
            object: self.object.traverse(g)?,
        })
    }
}

impl<'a, F> Object<'a, F> {
    pub fn traverse<F1, E>(
        &self,
        f: impl FnMut(usize, &F) -> Result<F1, E>,
    ) -> Result<Object<'a, F1>, E> {
        Ok(match self {
            Object::Untyped(obj) => Object::Untyped(obj.clone()),
            Object::Endpoint => Object::Endpoint,
            Object::Notification => Object::Notification,
            Object::CNode(obj) => Object::CNode(obj.clone()),
            Object::TCB(obj) => Object::TCB(obj.clone()),
            Object::IRQ(obj) => Object::IRQ(obj.clone()),
            Object::VCPU => Object::VCPU,
            Object::Frame(obj) => Object::Frame(obj.traverse(f)?),
            Object::PT(obj) => Object::PT(obj.clone()),
            Object::PD(obj) => Object::PD(obj.clone()),
            Object::PUD(obj) => Object::PUD(obj.clone()),
            Object::PGD(obj) => Object::PGD(obj.clone()),
            Object::ASIDPool(obj) => Object::ASIDPool(obj.clone()),
            Object::ArmIRQ(obj) => Object::ArmIRQ(obj.clone()),
        })
    }
}

impl<'a, F> object::Frame<'a, F> {
    pub fn traverse<F1, E>(
        &self,
        f: impl FnMut(usize, &F) -> Result<F1, E>,
    ) -> Result<object::Frame<'a, F1>, E> {
        Ok(object::Frame {
            size_bits: self.size_bits,
            paddr: self.paddr,
            fill: traverse_fill_entires(&self.fill, f)?,
        })
    }
}

fn traverse_fill_entires<'a, F, F1, E>(
    fill_entries: &[FillEntry<F>],
    mut f: impl FnMut(usize, &F) -> Result<F1, E>,
) -> Result<Indirect<'a, [FillEntry<F1>]>, E> {
    fill_entries
        .iter()
        .map(|entry| {
            Ok(FillEntry {
                range: entry.range.clone(),
                content: match &entry.content {
                    FillEntryContent::BootInfo(content_bootinfo) => {
                        FillEntryContent::BootInfo(*content_bootinfo)
                    }
                    FillEntryContent::Data(content_data) => {
                        FillEntryContent::Data(f(entry.range.len(), &content_data)?)
                    }
                },
            })
        })
        .collect::<Result<Box<[FillEntry<F1>]>, E>>()
        .map(Indirect::from_owned)
}

impl<'a, T> Indirect<'a, [T]> {
    fn traverse<T1, E>(&self, f: impl FnMut(&T) -> Result<T1, E>) -> Result<Indirect<'a, [T1]>, E> {
        self.iter()
            .map(f)
            .collect::<Result<Box<[T1]>, E>>()
            .map(Indirect::from_owned)
    }
}

impl<'a, N: Clone, F> Spec<'a, N, F> {
    pub fn traverse_fill_with_context<F1, E>(
        &self,
        f: impl FnMut(usize, &F) -> Result<F1, E>,
    ) -> Result<Spec<'a, N, F1>, E> {
        self.traverse(|_object, name| Ok(name.clone()), f)
    }

    pub fn traverse_fill<F1, E>(
        &self,
        mut f: impl FnMut(&F) -> Result<F1, E>,
    ) -> Result<Spec<'a, N, F1>, E> {
        self.traverse_fill_with_context(|_length, entry| f(entry))
    }
}

impl<'a, N, F: Clone> Spec<'a, N, F> {
    pub fn traverse_names_with_context<N1, E>(
        &self,
        f: impl FnMut(&Object<'a, F>, &N) -> Result<N1, E>,
    ) -> Result<Spec<'a, N1, F>, E> {
        self.traverse(f, |_length, entry| Ok(entry.clone()))
    }

    pub fn traverse_names<N1, E>(
        &self,
        mut f: impl FnMut(&N) -> Result<N1, E>,
    ) -> Result<Spec<'a, N1, F>, E> {
        self.traverse_names_with_context(|_object, name| f(name))
    }
}
