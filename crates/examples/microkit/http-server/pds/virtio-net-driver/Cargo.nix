#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, virtioDriversWith }:

mk {
  package.name = "microkit-http-server-example-virtio-net-driver";
  dependencies = {
    inherit (versions) log;

    virtio-drivers = virtioDriversWith [ "alloc" ];

    inherit (localCrates)
      sel4-microkit
      sel4-microkit-driver-adapters
      sel4-shared-memory
      sel4
      sel4-logging
      sel4-immediate-sync-once-cell
      sel4-shared-ring-buffer
      sel4-abstract-allocator
      sel4-virtio-hal-impl
      sel4-virtio-net
    ;
  };
}
