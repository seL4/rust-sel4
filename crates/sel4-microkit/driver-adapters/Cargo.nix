#
# Copyright 2023, Colias Group, LLC
# Copyright 2023, Galois, Inc.
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, serdeWith, smoltcpWith, authors }:

mk {
  package.name = "sel4-microkit-driver-adapters";
  package.authors = with authors; [
    nspin
    "Ben Hamlin <hamlinb@galois.com>"
  ];
  dependencies = {
    inherit (versions) log embedded-hal-nb heapless rtcc;
    serde = serdeWith [];
    smoltcp = smoltcpWith [];
    chrono = { version = versions.chrono; default-features = false; features = [ "serde" ]; };
  } // (with localCrates; {
    inherit
      sel4-driver-interfaces
      sel4-microkit
      sel4-microkit-message
      sel4-shared-ring-buffer
      sel4-shared-memory
      sel4-abstract-allocator
    ;
  });
  features = {
    # TODO
  };
}
