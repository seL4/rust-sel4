#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, mbedtlsWith, mbedtlsSysAutoWith, mbedtlsPlatformSupportWith }:

mk {
  package.name = "sel4-async-network-mbedtls";
  dependencies = {
    inherit (versions) log;
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
    rand = {
      version = "0.8.5";
      default-features = false;
      features = [
        "small_rng"
      ];
    };
    mbedtls = mbedtlsWith [];
    inherit (localCrates)
      sel4-async-network
      sel4-async-network-mbedtls-mozilla-ca-list
      # mbedtls
    ;
  };
}
