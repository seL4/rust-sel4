{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "tests-root-task-loader";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task-runtime
    sel4-platform-info
  ];
  dependencies = {
    fdt = "0.1.4";
  };
}
