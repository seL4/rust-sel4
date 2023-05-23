{ mk, localCrates }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" "capdl-spec" ];
  package.name = "capdl-loader-with-embedded-spec";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-logging
    capdl-types
    capdl-loader-core
    capdl-loader-with-embedded-spec-embedded-spec
    sel4-root-task-runtime
  ];
  nix.local.build-dependencies = with localCrates; [
    capdl-loader-with-embedded-spec-embedded-spec-validate
  ];
  dependencies = {
    sel4-root-task-runtime = { default-features = false; features = [ "single-threaded" ]; };
  };
  features = {
    deflate = [
      "capdl-loader-with-embedded-spec-embedded-spec/deflate"
      "capdl-loader-with-embedded-spec-embedded-spec-validate/deflate"
    ];
  };
}
