{ mk, localCrates }:

mk {
  package.name = "sel4-root-task-runtime";
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
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-panicking
    sel4-panicking-env
    sel4-runtime-common
    sel4-root-task-runtime-macros
  ];
  nix.meta.requirements = [ "sel4" ];
}
