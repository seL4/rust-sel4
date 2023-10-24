#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith }:

mk {
  package.name = "microkit-http-server-example-virtio-net-driver-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
}
