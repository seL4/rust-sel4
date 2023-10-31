//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use similar::{ChangeTag, TextDiff};

pub fn display_diff(a: &str, b: &str) -> String {
    let mut s = String::new();
    let d = TextDiff::from_lines(a, b);
    for change in d.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        s.push_str(&format!("{}{}", sign, change))
    }
    s
}
