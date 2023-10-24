#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, zerocopyWith }:

mk {
  package.name = "sel4-async-block-io-cpiofs";
  dependencies = {
    inherit (versions) log;
    zerocopy = zerocopyWith [ "derive" ];
    hex = { version = "0.4.3"; default-features = false; };
    lru = "0.10.0";
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
    inherit (localCrates)
      sel4-async-block-io
    ;
  };
}
