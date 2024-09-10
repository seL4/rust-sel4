//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cmp::Reverse;

// Determine whether rustc includes the following changes:
// - https://blog.rust-lang.org/2024/05/06/check-cfg.html
// - https://github.com/rust-lang/rust/pull/121598
// - https://github.com/rust-lang/rust/pull/126732

fn main() {
    let key = {
        let version_meta = rustc_version::version_meta().unwrap();
        let semver = version_meta.semver;
        let commit_date = order_date(version_meta.commit_date);
        (semver.major, semver.minor, semver.patch, commit_date)
    };
    let check_cfg_required = (1, 80, 0, order_date(Some("2024-05-05".to_owned())));
    let unwind_intrinsic_renamed = (1, 78, 0, order_date(Some("2024-02-28".to_owned())));
    let panic_info_message_stabilized = (1, 81, 0, order_date(Some("2024-07-01".to_owned())));
    if key >= check_cfg_required {
        println!("cargo:rustc-check-cfg=cfg(catch_unwind_intrinsic_still_named_try)");
        println!("cargo:rustc-check-cfg=cfg(panic_info_message_stable)");
    }
    if key < unwind_intrinsic_renamed {
        println!("cargo:rustc-cfg=catch_unwind_intrinsic_still_named_try");
    }
    if key >= panic_info_message_stabilized {
        println!("cargo:rustc-cfg=panic_info_message_stable");
    }
}

// no build date means more recent
fn order_date(date: Option<String>) -> Reverse<Option<Reverse<String>>> {
    Reverse(date.map(Reverse))
}
