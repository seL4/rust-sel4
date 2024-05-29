#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, verusSource }:

mk {
  package.name = "tests-root-task-verus-core";
  dependencies = {
    builtin = verusSource;
    builtin_macros = verusSource;
    vstd = verusSource // { default-features = false; };
  };
  package.metadata.verus = {
    verify = true;
  };
}
