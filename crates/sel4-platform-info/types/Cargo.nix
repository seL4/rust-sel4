#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, serdeWith }:

mk {
  package.name = "sel4-platform-info-types";
  dependencies = {
    serde = serdeWith [ "alloc" "derive" ] // {
      optional = true;
    };
  };
  features = {
    owned = [ "dep:serde" ];
  };
}
