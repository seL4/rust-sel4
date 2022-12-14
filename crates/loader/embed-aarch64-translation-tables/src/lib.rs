mod embed;
mod glue;
mod regions;
mod table;

pub use glue::{construct_and_embed_table, BlockDescriptor, Region, Regions};
pub use regions::{AbstractRegion, AbstractRegions};
pub use table::{AbstractEntry, MkLeafFnParams, RegionContent, PHYS_BOUNDS};
