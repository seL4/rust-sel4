{ mk, localCrates }:

mk {
  package.name = "sel4-capdl-initializer-with-embedded-spec-embedded-spec-validate";
  features = {
    deflate = [ "sel4-capdl-initializer-with-embedded-spec-embedded-spec/deflate" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-types
    sel4-capdl-initializer-with-embedded-spec-build-env
    sel4-capdl-initializer-with-embedded-spec-embedded-spec
  ];
  nix.meta.requirements = [ "linux" "capdl-spec" ];
}
