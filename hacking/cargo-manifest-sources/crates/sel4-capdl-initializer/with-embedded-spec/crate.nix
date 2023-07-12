{ mk, localCrates }:

mk {
  package.name = "sel4-capdl-initializer-with-embedded-spec";
  dependencies = {
    sel4-root-task = { default-features = false; features = [ "single-threaded" ]; };
  };
  features = {
    deflate = [
      "sel4-capdl-initializer-with-embedded-spec-embedded-spec/deflate"
      "sel4-capdl-initializer-with-embedded-spec-embedded-spec-validate/deflate"
    ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-core
    sel4-capdl-initializer-with-embedded-spec-embedded-spec
    sel4-capdl-initializer-types
    sel4
    sel4-logging
    sel4-root-task
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-capdl-initializer-with-embedded-spec-embedded-spec-validate
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" "capdl-spec" ];
}
