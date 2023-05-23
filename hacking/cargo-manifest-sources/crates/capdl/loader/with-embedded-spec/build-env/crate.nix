{ mk, localCrates, versions }:

mk {
  package.name = "capdl-loader-with-embedded-spec-build-env";
  nix.local.dependencies = with localCrates; [
    capdl-embed-spec
    capdl-types
  ];
  dependencies = {
    capdl-types.features = [ "alloc" "serde" ];
    inherit (versions) serde serde_json;
  };
  nix.meta.requirements = [ "linux" ];
}
