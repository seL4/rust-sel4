{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-config";
  nix.local.dependencies = with localCrates; [
    sel4-config-macros
  ];
}
