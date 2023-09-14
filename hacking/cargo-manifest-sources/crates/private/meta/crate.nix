{ mk, localCrates, versions }:

mk {
  package.name = "meta";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-config
    sel4-sys

    sel4-root-task

    sel4-microkit
    sel4-microkit-message
    sel4-microkit-message-types

    sel4-async-block-io
    sel4-async-block-io-cpiofs
    sel4-async-network
    sel4-async-request-statuses
    sel4-async-single-threaded-executor
    sel4-async-timers
    sel4-bounce-buffer-allocator
    sel4-externally-shared
    sel4-immediate-sync-once-cell
    sel4-immutable-cell
    sel4-logging
    sel4-shared-ring-buffer
    sel4-shared-ring-buffer-block-io
    sel4-shared-ring-buffer-block-io-types
    sel4-shared-ring-buffer-smoltcp
    sel4-sync
  ];
  dependencies = {
    inherit (versions) cfg-if log;
    sel4-externally-shared = { features = [ "alloc" "unstable" "very_unstable" ]; };
    sel4-root-task = { features = [ "full" ]; optional = true; };
    sel4-microkit = { features = [ "full" ]; optional = true; };
    sel4-microkit-message = { optional = true; };
    sel4-microkit-message-types = { optional = true; };
  };
  nix.local.target."cfg(not(target_thread_local))".dependencies = with localCrates; [
    sel4
  ];
  target."cfg(not(target_thread_local))".dependencies = {
    sel4 = { features = [ "single-threaded" ]; };
  };
  nix.local.target."cfg(not(target_arch = \"x86_64\"))".dependencies = with localCrates; [
    sel4-platform-info
  ];
  target."cfg(not(target_arch = \"x86_64\"))".dependencies = {
    sel4-platform-info = { optional = true; };
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
  nix.meta.requirements = [ "sel4" ];
}
