{ mk, localCrates, versions }:

mk {
  package.name = "sel4cp-http-server-example-virtio-net-driver";
  dependencies = rec {
    inherit (versions) log;

    virtio-drivers = {
      version = "0.5.0";
      default-features = false;
      features = [ "alloc" ];
    };

    sel4cp = { default-features = false; };
    sel4-externally-shared.features = [ "unstable" "alloc" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-logging
    sel4cp
    sel4-externally-shared
    sel4-sync
    sel4-immediate-sync-once-cell
    sel4-shared-ring-buffer
    sel4-bounce-buffer-allocator
    sel4cp-http-server-example-virtio-net-driver-interface-types
    sel4cp-http-server-example-virtio-hal-impl
  ];
}
