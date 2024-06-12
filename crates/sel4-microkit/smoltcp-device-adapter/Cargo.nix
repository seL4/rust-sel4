#
# Copyright 2023, Colias Group, LLC
# Copyright 2023, Galois, Inc.
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, smoltcpWith, serdeWith, authors }:

mk {
  package.name = "sel4-microkit-smoltcp-device-adapter";
  package.authors = with authors; [
    nspin
    "Ben Hamlin <hamlinb@galois.com>"
  ];
  dependencies = {
    inherit (versions) log;
    smoltcp = smoltcpWith [];
    serde = serdeWith [];
  } // (with localCrates; {
    inherit sel4-microkit-message;
    inherit sel4-driver-traits;
    sel4-microkit = sel4-microkit // { default-features = false; };
    sel4-bounce-buffer-allocator = sel4-bounce-buffer-allocator;
    sel4-externally-shared = sel4-externally-shared // { features = [ "unstable" ]; };
    sel4-shared-ring-buffer = sel4-shared-ring-buffer;
  });
}