{ mk, localCrates }:

mk {
  package.name = "sel4-root-task";
  dependencies = {
    sel4-runtime-common.features = [ "tls" "start" "static-heap" ];
  };
  features = {
    default = [
      "unwinding"
    ];
    full = [
      "default"
      "alloc"
    ];
    unwinding = [
      "sel4-panicking/unwinding"
      "sel4-runtime-common/unwinding"
    ];
    alloc = [
      "sel4-panicking/alloc"
    ];
    single-threaded = [
      "sel4/single-threaded"
    ];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-panicking
    sel4-panicking-env
    sel4-runtime-common
    sel4-root-task-macros
  ];
  nix.meta.requirements = [ "sel4" ];
}
