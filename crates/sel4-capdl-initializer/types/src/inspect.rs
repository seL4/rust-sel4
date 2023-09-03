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

    // HACKTMP
    pub fn page_table_level_for_riscv64sv39<'b: 'a>(
        &'b self,
        pt: &'a crate::object::PageTable<'a>,
    ) -> Option<usize> {
        let mut next_level = None;
        for (_, entry) in pt.entries() {
            let this_next_level = match entry {
                crate::PageTableEntry::Frame(cap) => {
                    let frame = self
                        .lookup_object::<&crate::object::Frame<'a, D, M>>(cap.object)
                        .unwrap();
                    Some(match frame.size_bits {
                        12 => 3,
                        21 => 2,
                        _ => panic!(),
                    })
                }
                crate::PageTableEntry::PageTable(cap) => {
                    let pt = self
                        .lookup_object::<&crate::object::PageTable>(cap.object)
                        .unwrap();
                    self.page_table_level_for_riscv64sv39(pt)
                }
            };
            if let Some(this_next_level) = this_next_level {
                if let Some(old_next_level) = next_level.replace(this_next_level) {
                    assert_eq!(old_next_level, this_next_level);
                }
            }
        }
        next_level.map(|next_level| next_level - 1)
    }

    // HACKTMP
    pub fn page_table_level_for_riscv64sv39_is_root<'b: 'a>(
        &'b self,
        pt: &'a crate::object::PageTable<'a>,
    ) -> Option<bool> {
        self.page_table_level_for_riscv64sv39(pt)
            .map(|level| level == 0)
    }
}
