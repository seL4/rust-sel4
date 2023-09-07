{ mk, localCrates, versions, postcardWith, serdeWith }:

mk {
  package.name = "sel4-kernel-loader-add-payload";
  dependencies = {
    sel4-kernel-loader-payload-types.features = [ "serde" ];
    sel4-config-generic-types.features = [ "serde" ];
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
      num
    ;
  };
  nix.local.dependencies = with localCrates; [
    sel4-kernel-loader-payload-types
    sel4-kernel-loader-config-types
    sel4-render-elf-with-data
    sel4-config-generic-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
