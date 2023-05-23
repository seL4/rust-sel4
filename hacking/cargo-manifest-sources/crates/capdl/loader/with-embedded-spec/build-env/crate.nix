{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "capdl-loader-with-embedded-spec-build-env";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    capdl-types
    capdl-embed-spec
  ];
  dependencies = {
    capdl-types.features = [ "alloc" "serde" ];
    inherit (versions) serde_json;
    inherit (versions) serde;
  };
}
