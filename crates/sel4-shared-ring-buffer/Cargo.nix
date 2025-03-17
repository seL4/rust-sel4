#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, zerocopyWith }:

mk {
  package.name = "sel4-shared-ring-buffer";
  dependencies = {
    inherit (versions) log;
    zerocopy = zerocopyWith [ "derive" ];
    sel4-shared-memory = localCrates.sel4-shared-memory // {
      features = [ "atomics" ];
    };
  };
}
