use crate::{NamedObject, Object, ObjectId, Spec};

impl<'a, N, F> Spec<'a, N, F> {
    pub fn num_objects(&self) -> usize {
        self.objects.len()
    }

    pub fn named_object(&self, obj_id: ObjectId) -> &NamedObject<'a, N, F> {
        &self.objects[obj_id]
    }

    pub fn name(&self, obj_id: ObjectId) -> &N {
        &self.named_object(obj_id).name
    }

    pub fn object(&self, obj_id: ObjectId) -> &Object<'a, F> {
        &self.named_object(obj_id).object
    }

    pub fn root_objects(&self) -> &[NamedObject<'a, N, F>] {
        &self.objects[self.root_objects.clone()]
    }

    pub fn named_objects(&self) -> impl Iterator<Item = &NamedObject<'a, N, F>> {
        self.objects.iter()
    }

    pub fn objects(&self) -> impl Iterator<Item = &Object<'a, F>> {
        self.named_objects()
            .map(|named_object| &named_object.object)
    }

    pub fn filter_objects<'b: 'a, O: TryFrom<&'a Object<'a, F>>>(
        &'b self,
    ) -> impl Iterator<Item = (ObjectId, O)> + 'b {
        self.objects()
            .enumerate()
            .filter_map(|(obj_id, obj)| Some((obj_id, O::try_from(obj).ok()?)))
    }

    pub fn filter_objects_with<'b: 'a, O: TryFrom<&'a Object<'a, F>>>(
        &'b self,
        f: impl 'a + Fn(&O) -> bool,
    ) -> impl Iterator<Item = (ObjectId, O)> + 'b {
        self.filter_objects().filter(move |(_, obj)| (&f)(obj))
    }

    pub fn lookup_object<'b: 'a, O: TryFrom<&'b Object<'a, F>>>(
        &'b self,
        obj_id: ObjectId,
    ) -> Result<O, O::Error> {
        self.object(obj_id).try_into()
    }
}
