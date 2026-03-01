#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "tests-root-task-verus-core";
  dependencies = {
    inherit (versions) verus_builtin verus_builtin_macros;
    vstd = { version = versions.vstd; } // { default-features = false; };
  };
  package.metadata.verus = {
    verify = true;
  };
}
