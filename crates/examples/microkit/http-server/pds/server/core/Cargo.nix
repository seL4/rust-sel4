#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, smoltcpWith, rustlsWith }:

mk {
  package.name = "microkit-http-server-example-server-core";
  dependencies = {
    inherit (versions) log webpki-roots;

    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "async-await"
        "alloc"
      ];
    };

    httparse = { version = "1.8.0"; default-features = false; };

    smoltcp = smoltcpWith [];

    rustls = rustlsWith [] // (localCrates.rustls or {});

    rustls-pemfile = { version = "2.0.0"; default-features = false; };

    inherit (localCrates)
      sel4-async-single-threaded-executor
      sel4-async-unsync
      sel4-async-time
      sel4-async-network
      sel4-async-io
      sel4-async-network-rustls
      sel4-async-network-rustls-utils
      sel4-panicking-env
      sel4-async-block-io
      sel4-async-block-io-fat
    ;
  };
}
