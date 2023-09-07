{ mk, localCrates, versions, postcardWith }:

mk {
  package.name = "sel4-capdl-initializer-add-spec";
  dependencies = {
    sel4-capdl-initializer-types.features = [ "std" "serde" "deflate" ];
    clap = "3.2.23";
    object = { version = versions.object; features = [ "all" ]; };
    postcard = postcardWith [ "alloc" ];
    inherit (versions)
      anyhow
      fallible-iterator
      serde_json
      num
    ;
  };
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-types
    sel4-render-elf-with-data
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
