//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::{NamedObject, Object, ObjectId, Spec};

impl<N, D, M> Spec<N, D, M> {
    pub fn num_objects(&self) -> usize {
        self.objects.len()
    }

    pub fn named_object(&self, obj_id: ObjectId) -> &NamedObject<N, D, M> {
        &self.objects[obj_id]
    }

    pub fn name(&self, obj_id: ObjectId) -> &N {
        &self.named_object(obj_id).name
    }

    pub fn object(&self, obj_id: ObjectId) -> &Object<D, M> {
        &self.named_object(obj_id).object
    }

    pub fn root_objects(&self) -> &[NamedObject<N, D, M>] {
        &self.objects[self.root_objects.clone()]
    }

    pub fn named_objects(&self) -> impl Iterator<Item = &NamedObject<N, D, M>> {
        self.objects.iter()
    }

    pub fn objects(&self) -> impl Iterator<Item = &Object<D, M>> {
        self.named_objects()
            .map(|named_object| &named_object.object)
    }

    pub fn filter_objects<'a, O: TryFrom<&'a Object<D, M>>>(
        &'a self,
    ) -> impl Iterator<Item = (ObjectId, O)> + 'a {
        self.objects()
            .enumerate()
            .filter_map(|(obj_id, obj)| Some((obj_id, O::try_from(obj).ok()?)))
    }

    pub fn filter_objects_with<'a, O: TryFrom<&'a Object<D, M>>>(
        &'a self,
        f: impl 'a + Fn(&O) -> bool,
    ) -> impl Iterator<Item = (ObjectId, O)> + 'a {
        self.filter_objects().filter(move |(_, obj)| (f)(obj))
    }

    pub fn lookup_object<'a, O: TryFrom<&'a Object<D, M>>>(
        &'a self,
        obj_id: ObjectId,
    ) -> Result<O, O::Error> {
        self.object(obj_id).try_into()
    }
}
