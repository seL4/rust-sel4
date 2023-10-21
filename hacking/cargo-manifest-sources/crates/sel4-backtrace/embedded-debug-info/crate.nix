#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-backtrace-embedded-debug-info";
  dependencies = {
    addr2line = { version = versions.addr2line; default-features = false; };
    object = { version = versions.object; default-features = false; features = [ "read" ]; };
  };
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-addr2line-context-helper
  ];
}
