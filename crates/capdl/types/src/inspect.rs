use crate::{ConcreteNamedObject, ConcreteObject, ConcreteSpec, Container, ObjectId};

impl<'a, T: Container<'a>, F: 'a, N: 'a> ConcreteSpec<'a, T, F, N> {
    pub fn num_objects(&self) -> usize {
        self.objects.as_slice().len()
    }

    pub fn named_object(&self, obj_id: ObjectId) -> &ConcreteNamedObject<'a, T, F, N> {
        &self.objects.as_slice()[obj_id]
    }

    pub fn name(&self, obj_id: ObjectId) -> &N {
        &self.named_object(obj_id).name
    }

    pub fn object(&self, obj_id: ObjectId) -> &ConcreteObject<'a, T, F> {
        &self.named_object(obj_id).object
    }

    pub fn named_objects(&self) -> impl Iterator<Item = &ConcreteNamedObject<'a, T, F, N>> {
        self.objects.as_slice().iter()
    }

    pub fn objects(&self) -> impl Iterator<Item = &ConcreteObject<'a, T, F>> {
        self.named_objects()
            .map(|named_object| &named_object.object)
    }

    pub fn filter_objects<'b: 'a, O: TryFrom<&'a ConcreteObject<'a, T, F>>>(
        &'b self,
    ) -> impl Iterator<Item = (ObjectId, O)> + 'b {
        self.objects()
            .enumerate()
            .filter_map(|(obj_id, obj)| Some((obj_id, O::try_from(obj).ok()?)))
    }

    pub fn lookup_object<'b: 'a, O: TryFrom<&'b ConcreteObject<'a, T, F>>>(
        &'b self,
        obj_id: ObjectId,
    ) -> Result<O, O::Error> {
        self.object(obj_id).try_into()
    }
}
