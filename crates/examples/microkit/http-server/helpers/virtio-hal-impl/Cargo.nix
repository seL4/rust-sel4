#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, virtioDriversWith }:

mk {
  package.name = "microkit-http-server-example-virtio-hal-impl";
  dependencies = {
    inherit (versions) log;
    virtio-drivers = virtioDriversWith [];
    inherit (localCrates)
      sel4-sync
      sel4-immediate-sync-once-cell
      sel4-externally-shared
      sel4-bounce-buffer-allocator
    ;
  };
}
