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
      sel4-microkit-message
      sel4
      sel4-logging
      sel4-immediate-sync-once-cell
      sel4-shared-ring-buffer
      sel4-shared-ring-buffer-block-io-types
      sel4-bounce-buffer-allocator
      sel4-virtio-hal-impl
      microkit-http-server-example-virtio-blk-driver-interface-types
    ;

    sel4-externally-shared = localCrates.sel4-externally-shared // { features = [ "unstable" ]; };
    sel4-microkit = localCrates.sel4-microkit // { default-features = false; };
  };
}
