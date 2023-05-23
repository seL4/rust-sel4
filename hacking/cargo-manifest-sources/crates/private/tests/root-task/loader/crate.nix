{ mk, localCrates }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "tests-root-task-loader";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task-runtime
    sel4-platform-info
  ];
  dependencies = {
    fdt = "0.1.4";
  };
}
