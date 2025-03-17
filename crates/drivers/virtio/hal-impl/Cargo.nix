#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, virtioDriversWith }:

mk {
  package.name = "sel4-virtio-hal-impl";
  dependencies = {
    inherit (versions)
      one-shot-mutex
    ;
    virtio-drivers = virtioDriversWith [];
    inherit (localCrates)
      sel4-immediate-sync-once-cell
      sel4-shared-memory
      sel4-bounce-buffer-allocator
    ;
  };
}
