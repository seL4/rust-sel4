{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "tests-capdl-http-server-components-test";
  dependencies = rec {
    inherit (versions) log;
    serde = serdeWith [ "alloc" "derive" ];
    dlmalloc = "0.2.3";

    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "async-await"
        "alloc"
        # "unstable"
        # "bilock"
      ];
    };

    smoltcp = {
      version = "0.9.1";
      default-features = false;
      features = [
        "log"
        "medium-ethernet" "medium-ip" "medium-ieee802154"
        "proto-ipv4" "proto-igmp" "proto-dhcpv4" "proto-ipv6" "proto-dns"
        "proto-ipv4-fragmentation" "proto-sixlowpan-fragmentation"
        "socket-raw" "socket-icmp" "socket-udp" "socket-tcp" "socket-dhcpv4" "socket-dns" "socket-mdns"
        "async"
      ];
    };

    # virtio-drivers = "0.5.0";
    virtio-drivers = {
      git = "https://github.com/nspin/virtio-drivers.git";
      rev = "409ee723c92adf309e825a7b87f53049707ed306";
      # default-features = false;

      # git = "https://github.com/rcore-os/virtio-drivers.git";
      # rev = "b003840720ac1f6970f2945f443a56ae28a68a3a";
    };
    # virtio-drivers = { default-features = false; };
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-sync
    sel4-logging
    sel4-simple-task-runtime
    sel4-simple-task-config-types
    sel4-async-single-threaded-executor
    sel4-bounce-buffer-allocator

    # virtio-drivers
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
