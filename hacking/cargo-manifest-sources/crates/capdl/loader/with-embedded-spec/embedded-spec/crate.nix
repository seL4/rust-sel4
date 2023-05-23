{ mk, localCrates, versions }:

mk {
  package.name = "capdl-loader-with-embedded-spec-embedded-spec";
  nix.local.dependencies = with localCrates; [
    capdl-types
  ];
  nix.local.build-dependencies = with localCrates; [
    capdl-types
    capdl-embed-spec
    capdl-loader-with-embedded-spec-build-env
    sel4-rustfmt-helper
  ];
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
  nix.meta.requirements = [ "capdl-spec" ];
}
