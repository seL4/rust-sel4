use std::borrow::Borrow;

use proc_macro2::{Ident, TokenStream};

mod embed;
mod table;

pub use table::{Region, PHYS_BOUNDS};

pub fn embed(ident: Ident, regions: impl Iterator<Item = impl Borrow<Region>>) -> TokenStream {
    table::Table::construct(regions).embed(ident)
}
