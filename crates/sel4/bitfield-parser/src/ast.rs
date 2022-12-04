pub type Ident = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub blocks: Vec<Entity<Block>>,
    pub tagged_unions: Vec<Entity<TaggedUnion>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity<T> {
    pub base: Base,
    pub inner: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base {
    pub base: usize,
    pub base_bits: usize,
    pub sign_extend: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub name: Ident,
    pub visible_order_spec: Option<Vec<Ident>>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub width: usize,
    pub field: Option<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Ident,
    pub is_high: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedUnion {
    pub name: Ident,
    pub tag_name: Ident,
    pub tag_slices: Vec<Ident>,
    pub classes: Vec<Class>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub width: usize,
    pub mask: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: Ident,
    pub values: Vec<usize>,
}
