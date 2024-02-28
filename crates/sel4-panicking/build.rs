//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cmp::Reverse;

// Determine whether rustc includes https://github.com/rust-lang/rust/pull/121598

fn main() {
    let version_meta = rustc_version::version_meta().unwrap();
    let semver = version_meta.semver;
    let commit_date = order_date(version_meta.commit_date);
    let key = (semver.major, semver.minor, semver.patch, commit_date);
    let first_with_change = (1, 78, 0, order_date(Some("2024-02-28".to_owned())));
    if key < first_with_change {
        println!("cargo:rustc-cfg=catch_unwind_intrinsic_still_named_try");
    }
}

// assume no build date means more recent
fn order_date(date: Option<String>) -> Reverse<Option<Reverse<String>>> {
    Reverse(date.map(Reverse))
}
