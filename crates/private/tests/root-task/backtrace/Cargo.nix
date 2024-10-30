#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-root-task-backtrace";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-backtrace-embedded-debug-info
    ;
    sel4-root-task = localCrates.sel4-root-task // { features = [ "unwinding" "alloc" ]; };
    sel4-backtrace-simple = localCrates.sel4-backtrace-simple // { features = [ "alloc" ]; };
    sel4-backtrace = localCrates.sel4-backtrace // { features = [ "full" ]; };
    sel4-backtrace-types = localCrates.sel4-backtrace-types // { features = [ "full" ]; };
  };
}
