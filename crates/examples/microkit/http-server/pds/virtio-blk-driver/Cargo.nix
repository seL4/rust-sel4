#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, virtioDriversWith}:

mk {
  package.name = "microkit-http-server-example-virtio-blk-driver";
  dependencies = {
    inherit (versions) log;

    virtio-drivers = virtioDriversWith [];

    inherit (localCrates)
      sel4-microkit
      sel4-microkit-message
      sel4-externally-shared
      sel4
      sel4-logging
      sel4-immediate-sync-once-cell
      sel4-shared-ring-buffer
      sel4-shared-ring-buffer-block-io-types
      sel4-bounce-buffer-allocator
      sel4-virtio-hal-impl
      sel4-virtio-blk
      sel4-driver-interfaces
      sel4-microkit-driver-adapters
    ;
  };
}
