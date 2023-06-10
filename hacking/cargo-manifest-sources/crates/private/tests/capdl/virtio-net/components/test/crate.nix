{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "tests-capdl-virtio-net-components-test";
  dependencies = {
    serde = serdeWith [ "alloc" "derive" ];
    # virtio-drivers = "0.4.0";
    virtio-drivers = {
      git = "https://github.com/rcore-os/virtio-drivers.git";
      rev = "b003840720ac1f6970f2945f443a56ae28a68a3a";
    };
    dlmalloc = "0.2.3";
    inherit (versions) log;
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-sync
    sel4-logging
    sel4-simple-task-runtime
    sel4-simple-task-config-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
