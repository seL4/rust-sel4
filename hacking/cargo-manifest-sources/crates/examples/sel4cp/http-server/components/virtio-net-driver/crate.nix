{ mk, localCrates, versions }:

mk {
  package.name = "tests-capdl-http-server-components-virtio-net-driver";
  dependencies = rec {
    inherit (versions) log;

    inherit (versions) zerocopy;
    bitflags = "1.3";
    volatile = "0.5.1";

    # virtio-drivers = {
    #   git = "https://github.com/nspin/virtio-drivers.git";
    #   rev = "409ee723c92adf309e825a7b87f53049707ed306"; # branch new-netdev
    #   # default-features = false; # disable "alloc"
    # };

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
    tests-capdl-http-server-components-virtio-net-driver-interface-types
  ];
}
