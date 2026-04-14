#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-test-sentinels-wrapper";
  dependencies = {
    inherit (versions)
      anyhow
    ;
  };
}
