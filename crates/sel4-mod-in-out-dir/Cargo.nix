#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-mod-in-out-dir";
  lib.proc-macro = true;
  dependencies = {
    inherit (versions) quote;
    syn = { version = versions.syn; features = [ "full" ]; };
  };
}
