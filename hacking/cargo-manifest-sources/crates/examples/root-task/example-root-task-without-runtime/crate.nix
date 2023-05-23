{ mk, localCrates, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "example-root-task-without-runtime";
  nix.local.dependencies = with localCrates; [
    sel4
  ];
  dependencies = {
    sel4.default-features = false;
    inherit (versions) cfg-if;
  };
}
