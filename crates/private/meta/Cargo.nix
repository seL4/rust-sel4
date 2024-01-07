#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "meta";
  dependencies = {
    inherit (versions) cfg-if log;

    inherit (localCrates)
      sel4
      sel4-config
      sel4-sys

      sel4-async-block-io
      sel4-async-block-io-cpiofs
      sel4-async-block-io-fat
      sel4-async-network
      sel4-async-single-threaded-executor
      sel4-async-time
      sel4-async-unsync
      sel4-bounce-buffer-allocator
      sel4-immediate-sync-once-cell
      sel4-immutable-cell
      sel4-logging
      sel4-one-ref-cell
      sel4-shared-ring-buffer
      sel4-shared-ring-buffer-block-io
      sel4-shared-ring-buffer-block-io-types
      sel4-shared-ring-buffer-bookkeeping
      sel4-shared-ring-buffer-smoltcp
      sel4-sync
    ;

    sel4-externally-shared = localCrates.sel4-externally-shared // { features = [ "unstable" ]; };
    sel4-root-task = localCrates.sel4-root-task // { features = [ "full" ]; optional = true; };
    sel4-microkit = localCrates.sel4-microkit // { features = [ "full" ]; optional = true; };
    sel4-microkit-message = localCrates.sel4-microkit-message // { optional = true; };
    sel4-microkit-message-types = localCrates.sel4-microkit-message-types // { optional = true; };
  };
  target."cfg(not(target_thread_local))".dependencies = {
    sel4 = localCrates.sel4 // { features = [ "single-threaded" ]; };
  };
  target."cfg(not(target_arch = \"x86_64\"))".dependencies = {
    sel4-platform-info = localCrates.sel4-platform-info  // { optional = true; };
  };
  features = {
    sel4-root-task = [
      "dep:sel4-root-task"
    ];
    sel4-microkit = [
      "dep:sel4-microkit"
      "sel4-microkit-message"
      "sel4-microkit-message-types"
    ];
  };
}
