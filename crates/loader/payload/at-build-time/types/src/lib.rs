#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

pub use loader_payload_types::{ImageInfo, PayloadInfo, Region};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadAtBuildTime<T> {
    pub info: PayloadInfo,
    pub data: Vec<Region<T>>,
}

impl<T> PayloadAtBuildTime<T> {
    pub fn traverse<U, E>(
        &self,
        mut f: impl FnMut(&T) -> Result<U, E>,
    ) -> Result<PayloadAtBuildTime<U>, E> {
        Ok(PayloadAtBuildTime {
            info: self.info.clone(),
            data: self
                .data
                .iter()
                .map(|region| region.traverse(&mut f))
                .collect::<Result<Vec<Region<U>>, E>>()?,
        })
    }
}
