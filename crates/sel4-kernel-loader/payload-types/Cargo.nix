#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-kernel-loader-payload-types";
  dependencies = {
    rkyv = { version = versions.rkyv; default-features = false; features = [ "alloc" "pointer_width_32" ]; };
    inherit (localCrates) sel4-platform-info-types;
  };
}
