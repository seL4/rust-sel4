{ mk, localCrates }:

mk {
  package.name = "tests-root-task-config";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
