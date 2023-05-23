{ mk, localCrates, versions }:

mk {
  package.name = "tests-root-task-panicking";
  dependencies = {
    inherit (versions) cfg-if;
  };
  features = {
    alloc = [ "sel4-root-task-runtime/alloc" ];
    panic-unwind = [];
    panic-abort = [];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task-runtime
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
