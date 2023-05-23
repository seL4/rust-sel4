{ mk, localCrates, postcardWith, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "capdl-loader-add-spec";
  nix.local.dependencies = with localCrates; [
    capdl-types
    capdl-loader-types
    sel4-render-elf-with-data
  ];
  dependencies = {
    capdl-types.features = [ "alloc" "serde" "deflate" ];
    object = { version = versions.object; features = [ "all" ]; };
    clap = "3.2.23";
    inherit (versions) anyhow;
    inherit (versions) fallible-iterator;
    inherit (versions) serde_json;
    postcard = postcardWith [ "alloc" ];
  };
}
