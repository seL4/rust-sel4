#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: MIT
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, versions, localCrates, smoltcpWith, virtioDriversWith, authors }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-virtio-net-driver-for-smoltcp";
  package.authors = with authors; [
    nspin
    "Runji Wang <wangrunji0408@163.com>"
  ];
  package.license = "MIT";
  dependencies = {
    inherit (versions) log;
    smoltcp = smoltcpWith [];
    virtio-drivers = virtioDriversWith [ "alloc" ];
    inherit (localCrates) sel4-driver-interfaces;
  };
}
