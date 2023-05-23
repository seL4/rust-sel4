{ mk, localCrates, serdeWith, coreLicense, meAsAuthor }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "tests-capdl-utcover-components-test";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-sync
    sel4-simple-task-runtime
    sel4-simple-task-config-types
  ];
  dependencies = {
    serde = serdeWith [ "alloc" "derive" ];
  };
}
