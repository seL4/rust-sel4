#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-async-block-io";
  dependencies = {
    inherit (versions) log;
    num_enum = { version = versions.num_enum; default-features = false; };
    futures = {
      version = versions.futures;
      default-features = false;
    };
    bytemuck = { version = versions.bytemuck; default-features = false; };
    gpt_disk_types = { version = versions.gpt_disk_types; features = [ "bytemuck" ]; };
    lru = { version = versions.lru; optional = true; };
  };
  features = {
    alloc = [ "futures/alloc" "lru" ];
    default = [ "alloc" ];
  };
}
