#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, zerocopyWith, serdeWith }:

mk {
  package.name = "sel4-simple-task-runtime-config-types";
  dependencies = {
    serde = serdeWith [ "derive" ] // { optional = true; };
    zerocopy = zerocopyWith [ "derive" ];
  };
  features = {
    alloc = [
      "serde?/alloc"
    ];
    serde = [
      "dep:serde"
    ];
  };
}
