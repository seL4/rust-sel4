{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer-embed-spec";
  dependencies = {
    sel4-capdl-initializer-types.features = [ "std" "deflate" ];
    hex = "0.4.3";
    syn = { version = versions.syn; features = [ "full" ]; };
    inherit (versions)
      proc-macro2
      quote
      serde
      serde_json
    ;
  };
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-types
  ];
}
