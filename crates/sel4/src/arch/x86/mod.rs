mod arch;
mod fault;
mod object;

pub(crate) mod top_level {
    pub use super::{
        arch::top_level::*,
        fault::Fault,
        object::{ObjectBlueprintArch, ObjectBlueprintX86, ObjectTypeArch, ObjectTypeX86},
    };
}
