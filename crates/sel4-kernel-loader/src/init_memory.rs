//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_kernel_loader_payload_types::ArchivedPayloadInfo;
use sel4_platform_info::PLATFORM_INFO;

use crate::this_image;

pub(crate) fn init() -> ArchivedPayloadInfo {
    let payload = this_image::get_payload();

    let own_footprint = this_image::get_user_image_bounds();

    log::debug!("Platform info: {PLATFORM_INFO:#x?}");
    log::debug!("Loader footprint: {own_footprint:#x?}");
    log::debug!("Payload info: {:#x?}", payload.info);
    log::debug!("Payload regions:");
    for region in payload.data.iter() {
        log::debug!(
            "    {:#x?} (filesz = {:#x?}, memsz = {:#x?})",
            region.addr.0,
            region.size.0,
            region.data.len()
        );
    }

    payload.sanity_check(&PLATFORM_INFO, own_footprint.clone());

    log::debug!("Copying payload data");
    unsafe {
        payload.copy_data_out();
    }

    payload.info.clone()
}
