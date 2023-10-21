#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, postcardWith, unwindingWith, serdeWith }:

mk {
  package.name = "sel4-backtrace";
  dependencies = {
    inherit (versions) cfg-if;
    unwinding = unwindingWith [] // { optional = true; };
    postcard = postcardWith [] // { optional = true; };
    serde = serdeWith [] // { optional = true; };
  };
  features = {
    alloc = [
      "sel4-backtrace-types/alloc"
    ];
    postcard = [
      "sel4-backtrace-types/postcard"
      "dep:postcard"
      "dep:serde"
    ];
    unwinding = [
      "dep:unwinding"
    ];
    full = [
      "alloc"
      "postcard"
      "unwinding"
    ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-types
  ];
}
