#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, serdeWith, smoltcpWith, ringWith }:

mk {
  package.name = "microkit-http-server-example-server";

  dependencies = {
    inherit (versions) log;

    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "async-await"
        "alloc"
      ];
    };

    smoltcp = smoltcpWith [
      "log"
    ];

    async-unsync = { version = versions.async-unsync; default-features = false; };

    inherit (localCrates)
      sel4
      sel4-logging
      sel4-immediate-sync-once-cell
      sel4-microkit-message
      sel4-microkit-smoltcp-device-adapter
      sel4-async-single-threaded-executor
      sel4-async-network
      sel4-async-time
      sel4-shared-ring-buffer-bookkeeping
      sel4-bounce-buffer-allocator
      sel4-shared-ring-buffer
      sel4-shared-ring-buffer-smoltcp
      sel4-shared-ring-buffer-block-io
      sel4-shared-ring-buffer-block-io-types
      sel4-async-block-io
      sel4-async-block-io-fat
      microkit-http-server-example-pl031-driver-interface-types
      microkit-http-server-example-sp804-driver-interface-types
      microkit-http-server-example-virtio-blk-driver-interface-types
    ;

    sel4-newlib = localCrates.sel4-newlib // {
      features = [
        "nosys"
        "all-symbols"
      ];
    };

    sel4-microkit = localCrates.sel4-microkit // { default-features = false; features = [ "alloc" ]; };
    sel4-externally-shared = localCrates.sel4-externally-shared // { features = [ "unstable" ]; };

    microkit-http-server-example-server-core = localCrates.microkit-http-server-example-server-core // {
      features = [
        # "debug"
      ];
    };
  };

  build-dependencies = {
    rcgen = "0.11.1";
  };
}
