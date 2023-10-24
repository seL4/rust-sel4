#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-root-task-macros";
  lib.proc-macro = true;
  dependencies = {
    syn = { version = versions.syn; features = [ "full" ]; };
    inherit (versions) proc-macro2 quote;
  };
}
