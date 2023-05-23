{ mk, localCrates, versions }:

mk {
  package.name = "example-root-task-without-runtime";
  dependencies = {
    sel4.default-features = false;
    inherit (versions) cfg-if;
  };
  nix.local.dependencies = with localCrates; [
    sel4
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
