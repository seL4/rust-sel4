{ mk, localCrates, serdeWith }:

mk {
  package.name = "tests-capdl-threads-components-test";
  dependencies = {
    serde = serdeWith [ "alloc" "derive" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-sync
    sel4-simple-task-runtime
    sel4-simple-task-config-types
  ];
}
