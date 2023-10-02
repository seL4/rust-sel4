{ mk, localCrates, versions, postcardWith }:

mk {
  package.name = "sel4-capdl-initializer-add-spec";
  dependencies = {
    sel4-capdl-initializer-types.features = [ "std" "serde" "deflate" ];
    object = { version = versions.object; features = [ "all" ]; };
    postcard = postcardWith [ "alloc" ];
    inherit (versions)
      anyhow
      fallible-iterator
      serde_json
      num
      clap
    ;
  };
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-types
    sel4-render-elf-with-data
  ];
}
