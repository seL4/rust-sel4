{ mk, localCrates }:

mk {
  package.name = "sel4cp-hello";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4cp
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
