#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith, versions }:

mk {
  package.name = "sel4-kernel-loader-payload-types";
  dependencies = {
    rkyv = { version = versions.rkyv; default-features = false; features = [ "alloc" "bytecheck" "pointer_width_32" ]; };
    serde = serdeWith [ "derive" ] // { optional = true; };
    heapless = { version = versions.heapless; features = [ "serde" ]; };
    num-traits = { version = versions.num-traits; default-features = false; };
    inherit (localCrates) sel4-platform-info-types;
  };
}
