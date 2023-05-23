{ mk, localCrates, postcardCommon, coreLicense, meAsAuthor, serdeWith, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "sel4-loader-add-payload";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4-loader-payload-types
    sel4-render-elf-with-data
    sel4-loader-config-types
  ];
  dependencies = {
    sel4-loader-payload-types.features = [ "serde" ];
    object = { version = versions.object; features = [ "all" ]; };
    clap = "3.2.23";
    inherit (versions) anyhow;
    inherit (versions) fallible-iterator;
    inherit (versions) serde_json;
    inherit (versions) serde_yaml;
    postcard = postcardCommon;
    inherit (versions) heapless;
    serde = serdeWith [ "alloc" "derive" ];
  };
}
