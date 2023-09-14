{ mk, localCrates }:

mk {
  package.name = "microkit-hello";
  nix.local.dependencies = with localCrates; [
    sel4-microkit
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
