{ mk, localCrates, versions }:

mk {
  package.name = "capdl-embed-spec";
  dependencies = {
    capdl-types.features = [ "std" "deflate" ];
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
    capdl-types
  ];
  nix.meta.requirements = [ "linux" ];
}
