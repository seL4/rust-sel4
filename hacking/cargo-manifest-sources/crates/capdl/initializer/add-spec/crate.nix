{ mk, localCrates, versions, postcardWith }:

mk {
  package.name = "capdl-initializer-add-spec";
  dependencies = {
    capdl-types.features = [ "alloc" "serde" "deflate" ];
    clap = "3.2.23";
    object = { version = versions.object; features = [ "all" ]; };
    postcard = postcardWith [ "alloc" ];
    inherit (versions)
      anyhow
      fallible-iterator
      serde_json
    ;
  };
  nix.local.dependencies = with localCrates; [
    capdl-initializer-types
    capdl-types
    sel4-render-elf-with-data
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
