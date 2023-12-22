#
# Copyright 2023, Colias Group, LLC
# Copyright 2023, Galois, Inc.
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, smoltcpWith, serdeWith, authors }:

mk {
  package.name = "sel4-hal-adapters";
  package.authors = with authors; [
    nspin
    "Ben Hamlin <hamlinb@galois.com>"
  ];
  dependencies = {
    inherit (versions) log;
    smoltcp = smoltcpWith [] // { optional = true; };
    serde = serdeWith [];
  } // (with localCrates; {
    inherit sel4-microkit-message;
    sel4-microkit = sel4-microkit // { default-features = false; };

    # smoltcp-phy deps
    sel4-bounce-buffer-allocator = sel4-bounce-buffer-allocator // { optional = true; };
    sel4-externally-shared = sel4-externally-shared // { optional = true; features = ["unstable"]; };
    sel4-shared-ring-buffer = sel4-shared-ring-buffer // { optional = true; };
  });
  features = {
    default = ["smoltcp-hal"];
    smoltcp-hal = [
      "smoltcp"
      "sel4-shared-ring-buffer"
      "sel4-externally-shared"
      "sel4-bounce-buffer-allocator"
    ];
  };
}
