//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::cmp::{Ordering, Reverse};

use crate::{PathSegment, Policy};

pub struct CargoManifestPolicy;

impl Policy for CargoManifestPolicy {
    fn compare_keys(&self, path: &[PathSegment], a: &str, b: &str) -> Ordering {
        let ranking = if path.len() == 0 {
            Ranking {
                front: &[
                    "package",
                    "lib",
                    "bin",
                    "features",
                    "dependencies",
                    "dev-dependencies",
                    "build-dependencies",
                    "workspace",
                    "profile",
                ],
                back: &[],
            }
        } else if path.len() == 1 && path[0].is_table_key("package") {
            Ranking {
                front: &["name", "version"],
                back: &["description"],
            }
        } else if path.len() >= 2
            && path[path.len() - 2]
                .as_table_key()
                .map(|k| k.ends_with("dependencies"))
                .unwrap_or(false)
        {
            Ranking {
                front: &[
                    "path",
                    "git",
                    "branch",
                    "tag",
                    "rev",
                    "version",
                    "registry",
                    "default-features",
                    "features",
                    "optional",
                ],
                back: &[],
            }
        } else if path.len() == 2 && path[0].is_table_key("target") {
            Ranking {
                front: &["dependencies", "dev-dependencies", "build-dependencies"],
                back: &[],
            }
        } else if path.len() == 1 && path[0].is_table_key("workspace") {
            Ranking {
                front: &[],
                back: &["members", "exclude"],
            }
        } else {
            Ranking {
                front: &[],
                back: &[],
            }
        };
        ranking.compare(a, b)
    }

    fn is_always_table(&self, path: &[PathSegment]) -> bool {
        path.len() <= 1
            || (path.len() <= 3 && path[0].is_table_key("target"))
            || (path.len() <= 3 && path[0].is_table_key("profile"))
    }

    fn is_always_array_of_tables(&self, path: &[PathSegment]) -> bool {
        path.len() == 2 && path[1].is_table_key("bin")
    }
}

struct Ranking<'a> {
    front: &'a [&'a str],
    back: &'a [&'a str],
}

impl<'a> Ranking<'a> {
    fn order<'b>(&self, a: &'b str) -> (Reverse<Option<Reverse<usize>>>, Option<usize>, &'b str) {
        (
            Reverse(self.front.iter().position(|s| s == &a).map(Reverse)),
            self.back.iter().position(|s| s == &a),
            a,
        )
    }

    fn compare(&self, a: &str, b: &str) -> Ordering {
        self.order(a).cmp(&self.order(b))
    }
}
