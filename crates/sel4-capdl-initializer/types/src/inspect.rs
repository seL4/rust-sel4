use crate::{NamedObject, Object, ObjectId, Spec};

impl<'a, N, D, M> Spec<'a, N, D, M> {
    pub fn num_objects(&self) -> usize {
        self.objects.len()
    }

    pub fn named_object(&self, obj_id: ObjectId) -> &NamedObject<'a, N, D, M> {
        &self.objects[obj_id]
    }

    pub fn name(&self, obj_id: ObjectId) -> &N {
        &self.named_object(obj_id).name
    }

    pub fn object(&self, obj_id: ObjectId) -> &Object<'a, D, M> {
        &self.named_object(obj_id).object
    }

    pub fn root_objects(&self) -> &[NamedObject<'a, N, D, M>] {
        &self.objects[self.root_objects.clone()]
    }

    pub fn named_objects(&self) -> impl Iterator<Item = &NamedObject<'a, N, D, M>> {
        self.objects.iter()
    }

    pub fn objects(&self) -> impl Iterator<Item = &Object<'a, D, M>> {
        self.named_objects()
            .map(|named_object| &named_object.object)
    }

    pub fn filter_objects<'b: 'a, O: TryFrom<&'a Object<'a, D, M>>>(
        &'b self,
    ) -> impl Iterator<Item = (ObjectId, O)> + 'b {
        self.objects()
            .enumerate()
            .filter_map(|(obj_id, obj)| Some((obj_id, O::try_from(obj).ok()?)))
    }

    pub fn filter_objects_with<'b: 'a, O: TryFrom<&'a Object<'a, D, M>>>(
        &'b self,
        f: impl 'a + Fn(&O) -> bool,
    ) -> impl Iterator<Item = (ObjectId, O)> + 'b {
        self.filter_objects().filter(move |(_, obj)| (f)(obj))
    }

    pub fn lookup_object<'b: 'a, O: TryFrom<&'b Object<'a, D, M>>>(
        &'b self,
        obj_id: ObjectId,
    ) -> Result<O, O::Error> {
        self.object(obj_id).try_into()
    }
}
