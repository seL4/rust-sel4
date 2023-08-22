{ mk, localCrates, versions, virtioDriversWith}:

mk {
  package.name = "sel4cp-http-server-example-virtio-blk-driver";
  dependencies = rec {
    inherit (versions) log;

    virtio-drivers = virtioDriversWith [];

    sel4-externally-shared.features = [ "unstable" ];
    sel4cp = { default-features = false; };
  };
  nix.local.dependencies = with localCrates; [
    sel4cp
    sel4
    sel4-sync
    sel4-logging
    sel4-immediate-sync-once-cell
    sel4-externally-shared
    sel4-shared-ring-buffer
    sel4-shared-ring-buffer-block-io-types
    sel4-bounce-buffer-allocator

    sel4cp-http-server-example-virtio-hal-impl
  ];
}
