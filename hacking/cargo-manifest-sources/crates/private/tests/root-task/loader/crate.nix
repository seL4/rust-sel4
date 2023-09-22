{ mk, localCrates }:

mk {
  package.name = "tests-root-task-loader";
  dependencies = {
    fdt = "0.1.4";
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task
    sel4-platform-info
  ];
}
