{ mk, localCrates }:

mk {
  package.name = "tests-root-task-backtrace";
  dependencies = {
    sel4-root-task.features = [ "alloc" "single-threaded" ];
    sel4-backtrace-simple.features = [ "alloc" ];
    sel4-backtrace.features = [ "full" ];
    sel4-backtrace-types.features = [ "full" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-backtrace
    sel4-backtrace-types
    sel4-backtrace-simple
    sel4-backtrace-embedded-debug-info
    sel4-root-task
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
