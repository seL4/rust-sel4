//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub use sel4_test_sentinels::indicate_success;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".sel4_test_kind")]
pub static sel4_test_kind_capdl: () = ();

#[doc(hidden)]
#[macro_export]
macro_rules! embed_file {
    ($section_name:literal, $path:literal) => {
        const _: () = {
            #[used]
            #[unsafe(no_mangle)]
            #[unsafe(link_section = $section_name)]
            pub static DATA: [u8; include_bytes!($path).len()] = *include_bytes!($path);
        };
    };
}

#[macro_export]
macro_rules! embed_capdl_script {
    ($path:literal) => {
        $crate::embed_file!(".capdl_script", $path);
    };
}
