{ mk, localCrates, versions }:

mk {
  package.name = "sel4cp-http-server-example-virtio-hal-impl";
  dependencies = rec {
    inherit (versions) log;

    virtio-drivers = {
      version = "0.5.0";
      default-features = false;
      features = [ "alloc" ];
    };

    sel4-externally-shared.features = [ "unstable" "alloc" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-externally-shared
    sel4-sync
    sel4-immediate-sync-once-cell
    sel4-bounce-buffer-allocator
  ];
}
