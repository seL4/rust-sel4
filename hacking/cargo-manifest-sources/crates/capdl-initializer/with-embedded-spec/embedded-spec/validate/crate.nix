{ mk, localCrates }:

mk {
  package.name = "capdl-initializer-with-embedded-spec-embedded-spec-validate";
  features = {
    deflate = [ "capdl-initializer-with-embedded-spec-embedded-spec/deflate" ];
  };
  nix.local.dependencies = with localCrates; [
    capdl-initializer-types
    capdl-initializer-with-embedded-spec-build-env
    capdl-initializer-with-embedded-spec-embedded-spec
  ];
  nix.meta.requirements = [ "linux" "capdl-spec" ];
}
