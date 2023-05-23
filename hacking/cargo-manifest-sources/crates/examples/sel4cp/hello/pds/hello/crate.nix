{ mk, localCrates }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4cp-hello";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4cp
  ];
  nix.meta.skip = true;
}
