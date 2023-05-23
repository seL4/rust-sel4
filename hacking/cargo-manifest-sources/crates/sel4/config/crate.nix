{ mk, localCrates }:

mk {
  package.name = "sel4-config";
  nix.local.dependencies = with localCrates; [
    sel4-config-macros
  ];
  nix.meta.requirements = [ "sel4" ];
}
