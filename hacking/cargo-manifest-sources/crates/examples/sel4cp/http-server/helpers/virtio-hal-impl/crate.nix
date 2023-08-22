{ mk, localCrates, versions, virtioDriversWith }:

mk {
  package.name = "sel4cp-http-server-example-virtio-hal-impl";
  dependencies = rec {
    inherit (versions) log;

    virtio-drivers = virtioDriversWith [];
  };
  nix.local.dependencies = with localCrates; [
    sel4-sync
    sel4-immediate-sync-once-cell
    sel4-externally-shared
    sel4-bounce-buffer-allocator
  ];
}
