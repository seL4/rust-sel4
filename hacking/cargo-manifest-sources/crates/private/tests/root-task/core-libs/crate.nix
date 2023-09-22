{ mk, localCrates }:

mk {
  package.name = "tests-root-task-core-libs";
  nix.local.dependencies = with localCrates; [
    sel4-root-task
    sel4-sys
    sel4-config
    sel4
  ];
}
