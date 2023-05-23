{ mk, localCrates, postcardWith, versions }:

mk {
  package.name = "capdl-loader";
  dependencies = {
    capdl-types.features = [ "alloc" "serde" "deflate" ];
    postcard = postcardWith [ "alloc" ];
    sel4-root-task-runtime = { default-features = false; features = [ "alloc" "single-threaded" ]; };
    inherit (versions) log;
  };
  # features = {
  #   deflate = [
  #     "capdl-types/deflate"
  #   ];
  # };
  nix.local.dependencies = with localCrates; [
    capdl-loader-core
    capdl-loader-types
    capdl-types
    sel4
    sel4-dlmalloc
    sel4-logging
    sel4-root-task-runtime
    sel4-sync
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" "capdl-spec" ];
}
