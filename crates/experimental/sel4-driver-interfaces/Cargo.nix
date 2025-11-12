#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, serdeWith }:

mk {
  package.name = "sel4-driver-interfaces";
  dependencies = {
    inherit (versions) embedded-hal-nb rtcc heapless lock_api;
    serde = serdeWith [ "derive" ];
  };
}
