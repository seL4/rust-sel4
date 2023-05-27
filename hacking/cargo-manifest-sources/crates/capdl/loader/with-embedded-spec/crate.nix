{ mk, localCrates }:

mk {
  package.name = "capdl-loader-with-embedded-spec";
  dependencies = {
    sel4-root-task = { default-features = false; features = [ "single-threaded" ]; };
  };
  features = {
    deflate = [
      "capdl-loader-with-embedded-spec-embedded-spec/deflate"
      "capdl-loader-with-embedded-spec-embedded-spec-validate/deflate"
    ];
  };
  nix.local.dependencies = with localCrates; [
    capdl-loader-core
    capdl-loader-with-embedded-spec-embedded-spec
    capdl-types
    sel4
    sel4-logging
    sel4-root-task
  ];
  nix.local.build-dependencies = with localCrates; [
    capdl-loader-with-embedded-spec-embedded-spec-validate
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" "capdl-spec" ];
}
