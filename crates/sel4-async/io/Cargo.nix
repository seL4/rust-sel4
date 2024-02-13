#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-async-io";
  dependencies = {
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
  };
}
