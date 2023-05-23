{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-root-task-runtime";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-panicking
    sel4-panicking-env
    sel4-runtime-common
    sel4-root-task-runtime-macros
  ];
  dependencies = {
    sel4-runtime-common.features = [ "tls" "unwinding" "start" "static-heap" ];
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
    ];
    alloc = [
      "sel4-panicking/alloc"
    ];
    single-threaded = [
      "sel4/single-threaded"
    ];
  };
}
