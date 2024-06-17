#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, virtioDriversWith }:

mk {
  package.name = "sel4-virtio-blk";
  dependencies = {
    inherit (versions) log;
    virtio-drivers = virtioDriversWith [];
    inherit (localCrates) sel4-driver-interfaces;
  };
}
