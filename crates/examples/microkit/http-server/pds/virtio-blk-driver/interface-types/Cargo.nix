#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, serdeWith }:

mk {
  package.name = "microkit-http-server-example-virtio-blk-driver-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
}
