#![feature(associated_type_defaults)]

mod embed;
mod glue;
mod regions;
mod scheme;
mod table;

pub use glue::{Region, Regions, RegionsBuilder};
pub use regions::{AbstractRegion, AbstractRegions, AbstractRegionsBuilder};
pub use scheme::{Scheme, SchemeHelpers, *};
pub use table::{LeafLocation, MkLeafFn, RegionContent, Table};
