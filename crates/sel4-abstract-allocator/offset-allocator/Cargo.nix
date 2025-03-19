#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, offsetAllocatorSource }:

mk {
  package.name = "sel4-abstract-allocator-offset-allocator";
  dependencies = {
    offset-allocator = offsetAllocatorSource;
    inherit (localCrates)
      sel4-abstract-allocator
      # offset-allocator
    ;
  };
}
