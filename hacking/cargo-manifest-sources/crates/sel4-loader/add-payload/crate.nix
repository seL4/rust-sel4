{ mk, localCrates, versions, postcardWith, serdeWith }:

mk {
  package.name = "sel4-loader-add-payload";
  dependencies = {
    sel4-loader-payload-types.features = [ "serde" ];
    object = { version = versions.object; features = [ "all" ]; };
    clap = "3.2.23";
    postcard = postcardWith [ "alloc" ];
    serde = serdeWith [ "alloc" "derive" ];
    inherit (versions)
      anyhow
      fallible-iterator
      serde_json
      serde_yaml
      heapless
    ;
  };
  nix.local.dependencies = with localCrates; [
    sel4-loader-payload-types
    sel4-render-elf-with-data
    sel4-loader-config-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
