{ mk, localCrates, versions }:

mk {
  package.name = "capdl-initializer-with-embedded-spec-embedded-spec";
  dependencies = {
    capdl-types.features = [ "borrowed-indirect" ];
  };
  build-dependencies = {
    inherit (versions) serde_json;
  };
  features = {
    deflate = [
      "capdl-types/deflate"
    ];
  };
  nix.local.dependencies = with localCrates; [
    capdl-types
  ];
  nix.local.build-dependencies = with localCrates; [
    capdl-embed-spec
    capdl-initializer-with-embedded-spec-build-env
    capdl-types
    sel4-rustfmt-helper
  ];
  nix.meta.requirements = [ "capdl-spec" ];
}
