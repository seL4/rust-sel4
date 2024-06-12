#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, smoltcpWith, virtioDriversWith }:

mk {
  package.name = "sel4-virtio-net-driver-for-smoltcp";
  dependencies = {
    inherit (versions) log;
    smoltcp = smoltcpWith [];
    virtio-drivers = virtioDriversWith [ "alloc" ];
    inherit (localCrates) sel4-driver-traits;
  };
}
