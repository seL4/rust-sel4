use alloc::vec::Vec;

use crate::{
    ConcreteSpec, Container, ContainerType, FillEntry, FillEntryContent, Spec, VecContainer,
};

impl<'a, T: Container<'a>, F, N> ConcreteSpec<'a, T, F, N> {
    pub fn traverse<F1, N1, E>(
        &self,
        mut f: impl FnMut(usize, &F) -> Result<F1, E>,
        mut g: impl FnMut(&N) -> Result<N1, E>,
    ) -> Result<ConcreteSpec<'a, VecContainer, F1, N1>, E> {
        Ok(Spec {
            objects: self.objects.traverse(|named_object| {
                named_object.traverse_simple(
                    &mut g,
                    |cap_table| Ok(cap_table.to_vec()),
                    |fill_entries| {
                        Ok(ContainerType(
                            fill_entries
                                .as_slice()
                                .iter()
                                .map(|entry| {
                                    Ok(FillEntry {
                                        range: entry.range.clone(),
                                        content: match &entry.content {
                                            FillEntryContent::BootInfo(content_bootinfo) => {
                                                FillEntryContent::BootInfo(content_bootinfo.clone())
                                            }
                                            FillEntryContent::Data(content_data) => {
                                                FillEntryContent::Data(f(
                                                    entry.range.end - entry.range.start,
                                                    &content_data,
                                                )?)
                                            }
                                        },
                                    })
                                })
                                .collect::<Result<Vec<FillEntry<F1>>, E>>()?,
                        ))
                    },
                )
            })?,
            irqs: self.irqs.to_vec(),
            asid_slots: self.asid_slots.to_vec(),
        })
    }
}

impl<'a, T: Container<'a>, F, N: Clone> ConcreteSpec<'a, T, F, N> {
    pub fn traverse_fill_with_context<F1, E>(
        &self,
        f: impl FnMut(usize, &F) -> Result<F1, E>,
    ) -> Result<ConcreteSpec<'a, VecContainer, F1, N>, E> {
        self.traverse(f, |name| Ok(name.clone()))
    }

    pub fn traverse_fill<F1, E>(
        &self,
        mut f: impl FnMut(&F) -> Result<F1, E>,
    ) -> Result<ConcreteSpec<'a, VecContainer, F1, N>, E> {
        self.traverse_fill_with_context(|_, entry| f(entry))
    }
}

impl<'a, T: Container<'a>, F: Clone, N> ConcreteSpec<'a, T, F, N> {
    pub fn traverse_names<N1, E>(
        &self,
        f: impl FnMut(&N) -> Result<N1, E>,
    ) -> Result<ConcreteSpec<'a, VecContainer, F, N1>, E> {
        self.traverse(|_, entry| Ok(entry.clone()), f)
    }
}

impl<'a, T: Container<'a>, F: Clone, N: Clone> ConcreteSpec<'a, T, F, N> {
    pub fn to_vec(&self) -> ConcreteSpec<'a, VecContainer, F, N> {
        self.traverse(
            |_, entry| Ok::<_, !>(entry.clone()),
            |name| Ok(name.clone()),
        )
        .into_ok()
    }
}

impl<'a, T: Container<'a>, A> ContainerType<'a, T, A> {
    pub fn traverse<B, E>(
        &self,
        f: impl FnMut(&A) -> Result<B, E>,
    ) -> Result<ContainerType<'a, VecContainer, B>, E> {
        Ok(ContainerType(
            self.as_slice()
                .iter()
                .map(f)
                .collect::<Result<Vec<B>, E>>()?,
        ))
    }
}

impl<'a, T: Container<'a>, A: Clone> ContainerType<'a, T, A> {
    pub fn to_vec(&self) -> ContainerType<'a, VecContainer, A> {
        self.traverse(|x| Ok::<_, !>(x.clone())).into_ok()
    }
}
