#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-newlib";
  features = {
    default = [ "detect-libc" ];
    detect-libc = [];
    nosys = [];
    _exit = [];
    __trunctfdf2 = [];
    _sbrk = [];
    _write = [];
    all-symbols = [
      "_exit"
      "_sbrk"
      "_write"
      "__trunctfdf2"
    ];
  };
  dependencies = {
    inherit (versions) log;
    inherit (localCrates)
      sel4-immediate-sync-once-cell
    ;
    sel4-panicking-env = localCrates.sel4-panicking-env // { optional = true; };
  };
  build-dependencies = {
    cc = "1.0.82";
  };
}
