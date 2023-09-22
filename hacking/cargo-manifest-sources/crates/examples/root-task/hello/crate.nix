{ mk, localCrates }:

mk {
  package.name = "hello";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task
  ];
}
