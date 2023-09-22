{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer-with-embedded-spec-embedded-spec";
  dependencies = {
    sel4-capdl-initializer-types.features = [ "borrowed-indirect" ];
  };
  build-dependencies = {
    inherit (versions) serde_json;
  };
  features = {
    deflate = [
      "sel4-capdl-initializer-types/deflate"
    ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-types
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-capdl-initializer-embed-spec
    sel4-capdl-initializer-with-embedded-spec-build-env
    sel4-capdl-initializer-types
    sel4-rustfmt-helper
  ];
}
