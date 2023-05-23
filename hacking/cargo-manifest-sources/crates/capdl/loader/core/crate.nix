{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "capdl-loader-core";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4
    capdl-types
  ];
  dependencies = {
    inherit (versions) log;
    capdl-types.features = [ "sel4" ];
  };
}
