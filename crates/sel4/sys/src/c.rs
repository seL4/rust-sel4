use crate::bf::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// HACK
// Handle anonymous enums, with sanity checks.
pub use _bindgen_ty_1::seL4_MsgMaxLength;
pub use _bindgen_ty_2 as seL4_RootCapSlot;
const __ROOT_CAP_SLOT_SANITY_CHECK: seL4_RootCapSlot::Type = seL4_RootCapSlot::seL4_CapNull;
