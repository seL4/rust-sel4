{ mk, localCrates, versions }:

mk {
  package.name = "capdl-initializer-with-embedded-spec-embedded-spec";
  dependencies = {
    capdl-initializer-types.features = [ "borrowed-indirect" ];
  };
  build-dependencies = {
    inherit (versions) serde_json;
  };
  features = {
    deflate = [
      "capdl-initializer-types/deflate"
    ];
  };
  nix.local.dependencies = with localCrates; [
    capdl-initializer-types
  ];
  nix.local.build-dependencies = with localCrates; [
    capdl-initializer-embed-spec
    capdl-initializer-with-embedded-spec-build-env
    capdl-initializer-types
    sel4-rustfmt-helper
  ];
  nix.meta.requirements = [ "capdl-spec" ];
}
