//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cmp::Reverse;

// Determine whether rustc includes https://github.com/rust-lang/rust/pull/122305

fn main() {
    let key = {
        let version_meta = rustc_version::version_meta().unwrap();
        let semver = version_meta.semver;
        let commit_date = order_date(version_meta.commit_date);
        (semver.major, semver.minor, semver.patch, commit_date)
    };
    let check_cfg_required = (1, 80, 0, order_date(Some("2024-05-05".to_owned())));
    let target_spec_has_metadata = (1, 78, 0, order_date(Some("2024-03-15".to_owned())));
    if key >= check_cfg_required {
        println!("cargo:rustc-check-cfg=cfg(target_spec_has_metadata)");
    }
    if key >= target_spec_has_metadata {
        println!("cargo:rustc-cfg=target_spec_has_metadata");
    }
}

// no build date means more recent
fn order_date(date: Option<String>) -> Reverse<Option<Reverse<String>>> {
    Reverse(date.map(Reverse))
}
