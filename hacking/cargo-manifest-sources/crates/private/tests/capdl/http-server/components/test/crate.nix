{ mk, localCrates, versions, serdeWith, smoltcpWith }:

mk {
  package.name = "tests-capdl-http-server-components-test";
  dependencies = rec {
    inherit (versions) log;
    serde = serdeWith [ "alloc" "derive" ];

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

    smoltcp = smoltcpWith [];

    virtio-drivers = {
      version = "0.5.0";
    };

    # virtio-drivers = {
    #   git = "https://github.com/nspin/virtio-drivers.git";
    #   rev = "409ee723c92adf309e825a7b87f53049707ed306"; # branch new-netdev
    #   # default-features = false; # disable "alloc"
    # };

    httparse = { version = "1.8.0"; default-features = false; };
    cpio_reader = "0.1.1";

    tock-registers = "0.8.1";

    # embedded-exfat = { version = "0.2.4"; default-features = false; features = [ "async" ]; };
    embedded-exfat = {
      # version = "0.2.4";
      git = "https://github.com/qiuchengxuan/exfat";
      rev = "64a72a9e260596a936ffed09e329b32333c42d92";
      default-features = false;
      features = [ "async" ];
    };
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-sync
    sel4-logging
    sel4-simple-task-runtime
    sel4-simple-task-config-types
    sel4-async-single-threaded-executor
    sel4-async-network
    sel4-async-timers
    sel4-bounce-buffer-allocator
    tests-capdl-http-server-components-test-sp804-driver

    # virtio-drivers
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
