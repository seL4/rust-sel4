#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, objectWith }:

mk {
  package.name = "sel4-backtrace-embedded-debug-info";
  dependencies = {
    addr2line = { version = versions.addr2line; default-features = false; };
    object = objectWith [];
    inherit (localCrates)
      sel4-backtrace-addr2line-context-helper
      sel4-phdrs
      sel4-phdrs-patched
    ;
  };
}
