{ mk, localCrates }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "example-root-task";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task-runtime
  ];
}
