#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, zerocopyWith, serdeWith }:

mk {
  package.name = "sel4-simple-task-runtime-config-types";
  dependencies = {
    serde = serdeWith [ "derive" "alloc" ] // { optional = true; };
    rkyv = {
      version = versions.rkyv;
      default-features = false;
      features = [ "alloc" "bytecheck" "pointer_width_32" ];
    };
  };
}
