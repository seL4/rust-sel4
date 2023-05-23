{ mk, localCrates, postcardWith, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" "capdl-spec" ];
  package.name = "capdl-loader";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-logging
    capdl-types
    capdl-loader-core
    sel4-root-task-runtime
    sel4-dlmalloc
    sel4-sync
    capdl-loader-types
  ];
  dependencies = {
    inherit (versions) log;
    sel4-root-task-runtime = { default-features = false; features = [ "alloc" "single-threaded" ]; };
    capdl-types.features = [
      "alloc" "serde"
      "deflate"
    ];
    postcard = postcardWith [ "alloc" ];
  };
  # features = {
  #   deflate = [
  #     "capdl-types/deflate"
  #   ];
  # };
}
