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
    detect-libc = [ "cc" ];
    nosys = [];
    _exit = [];
    _write = [];
    errno = [];
    all-symbols = [
      "_exit"
      "_write"
      "errno"
    ];
  };
  dependencies = {
    inherit (localCrates) sel4-panicking-env;
    log = { version = versions.log; optional = true; };
  };
  build-dependencies = {
    cc = { version = "1.0.82"; optional = true; };
  };
}
