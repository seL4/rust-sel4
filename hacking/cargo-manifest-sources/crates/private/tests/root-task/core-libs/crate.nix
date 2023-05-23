{ mk, localCrates }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "tests-root-task-core-libs";
  nix.local.dependencies = with localCrates; [
    sel4-root-task-runtime
    sel4-sys
    sel4-config
    sel4
  ];
}
