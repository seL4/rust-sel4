#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-shared-ring-buffer-bookkeeping";
  dependencies = {
    async-unsync = { version = versions.async-unsync; default-features = false; optional = true; };
  };
}
