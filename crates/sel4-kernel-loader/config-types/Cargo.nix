#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, serdeWith }:

mk {
  package.name = "sel4-kernel-loader-config-types";
  dependencies = {
    serde = serdeWith [ "derive" ];
  };
}
