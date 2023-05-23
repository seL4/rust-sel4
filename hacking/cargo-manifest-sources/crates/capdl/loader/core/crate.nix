{ mk, localCrates, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "capdl-loader-core";
  nix.local.dependencies = with localCrates; [
    sel4
    capdl-types
  ];
  dependencies = {
    inherit (versions) log;
    capdl-types.features = [ "sel4" ];
  };
}
