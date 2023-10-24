#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-simple-task-runtime-macros";
  lib.proc-macro = true;
  dependencies = {
    inherit (versions) proc-macro2 quote;
    syn = { version = versions.syn; features = [ "full" ]; };
  };
}
