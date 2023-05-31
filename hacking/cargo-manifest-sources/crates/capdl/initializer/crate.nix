{ mk, localCrates, postcardWith, versions }:

mk {
  package.name = "capdl-initializer";
  dependencies = {
    capdl-types.features = [ "alloc" "serde" "deflate" ];
    postcard = postcardWith [ "alloc" ];
    sel4-root-task = { default-features = false; features = [ "alloc" "single-threaded" ]; };
    inherit (versions) log;
  };
  # features = {
  #   deflate = [
  #     "capdl-types/deflate"
  #   ];
  # };
  nix.local.dependencies = with localCrates; [
    capdl-initializer-core
    capdl-initializer-types
    capdl-types
    sel4
    sel4-dlmalloc
    sel4-logging
    sel4-root-task
    sel4-sync
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" "capdl-spec" ];
}
