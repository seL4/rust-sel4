{ mk, localCrates }:

mk {
  package.name = "capdl-loader-with-embedded-spec-embedded-spec-validate";
  features = {
    deflate = [ "capdl-loader-with-embedded-spec-embedded-spec/deflate" ];
  };
  nix.local.dependencies = with localCrates; [
    capdl-types
    capdl-loader-with-embedded-spec-build-env
    capdl-loader-with-embedded-spec-embedded-spec
  ];
  nix.meta.requirements = [ "linux" "capdl-spec" ];
}
