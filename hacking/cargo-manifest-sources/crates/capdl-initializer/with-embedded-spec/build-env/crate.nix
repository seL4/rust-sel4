{ mk, localCrates, versions }:

mk {
  package.name = "capdl-initializer-with-embedded-spec-build-env";
  nix.local.dependencies = with localCrates; [
    capdl-initializer-embed-spec
    capdl-initializer-types
  ];
  dependencies = {
    capdl-initializer-types.features = [ "std" "serde" ];
    inherit (versions) serde serde_json;
  };
  nix.meta.requirements = [ "linux" ];
}
