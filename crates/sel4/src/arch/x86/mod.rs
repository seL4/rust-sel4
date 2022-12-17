mod arch;
mod object;
mod fault;

pub(crate) mod top_level {
    pub use super::{
        object::{ObjectBlueprintArch, ObjectTypeArch, ObjectBlueprintX86, ObjectTypeX86},
        fault::Fault,
        arch::top_level::*,
    };
}
