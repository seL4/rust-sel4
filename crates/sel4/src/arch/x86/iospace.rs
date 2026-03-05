// use sel4_config::sel4_cfg;

// #[sel4_cfg(IOMMU)]
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub enum VtdTranslationTableObjectType {
//     VtdPml4,
//     VtdPdpt,
//     VtdPd,
//     VtdPt,
// }

// const VTD_PAGE_TABLE_ENTRY_BIT: u32 = 3;
// const VTD_PAGE_TABLE_ENTRY_INDEX: u32 = sel4_sys::seL4_IOPageTableBits - VTD_PAGE_TABLE_ENTRY_BIT;

// #[sel4_cfg(IOMMU)]
// impl VtdTranslationTableObjectType {
//     pub const fn index_bits(&self) -> u32 {
//         VTD_PAGE_TABLE_ENTRY_INDEX
//     }
// }
