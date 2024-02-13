#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, smoltcpWith }:

mk {
  package.name = "sel4-async-network";
  dependencies = {
    inherit (localCrates) sel4-async-io;
    inherit (versions) log;
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
    smoltcp = smoltcpWith [
      "async"
      "alloc"
      # "verbose"
    ];
  };
}
