#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk rec {
  package.name = "sel4-async-single-threaded-executor";
  package.license = "MIT OR Apache-2.0";
  nix.reuseFrontmatterArgs.licenseID = package.license;
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
