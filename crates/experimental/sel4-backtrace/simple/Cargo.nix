#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-backtrace-simple";
  dependencies = {
    inherit (localCrates) sel4-panicking-env;
    sel4-backtrace = localCrates.sel4-backtrace // { features = [ "postcard" "unwinding" ]; };
  };
  features = {
    alloc = [
      "sel4-backtrace/alloc"
    ];
  };
}
