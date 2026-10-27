#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, objectWith }:

mk {
  package.name = "sel4-backtrace-cli";
  dependencies = {
    inherit (versions) hex;
    object = objectWith [ "compression" ];
    clap = { version = versions.clap; features = [ "derive" ]; };
    inherit (localCrates) sel4-backtrace-addr2line-context-helper;
    sel4-backtrace-types = localCrates.sel4-backtrace-types // { features = [ "full" ]; };
  };
}
