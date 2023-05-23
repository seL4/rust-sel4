{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "linux" "capdl-spec" ];
  package.name = "capdl-loader-with-embedded-spec-embedded-spec-validate";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    capdl-types
    capdl-loader-with-embedded-spec-build-env
    capdl-loader-with-embedded-spec-embedded-spec
  ];
  features = {
    deflate = [ "capdl-loader-with-embedded-spec-embedded-spec/deflate" ];
  };
}
