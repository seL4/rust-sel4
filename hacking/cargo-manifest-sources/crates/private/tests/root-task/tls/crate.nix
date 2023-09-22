{ mk, localCrates }:

mk {
  package.name = "tests-root-task-tls";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task
  ];
}
