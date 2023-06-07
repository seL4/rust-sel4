{ mk, localCrates, versions }:

mk {
  package.name = "capdl-initializer-core";
  nix.local.dependencies = with localCrates; [
    capdl-initializer-types
    sel4
  ];
  dependencies = {
    capdl-initializer-types.features = [ "sel4" ];
    inherit (versions) log;
  };
  nix.meta.requirements = [ "sel4" ];
}
