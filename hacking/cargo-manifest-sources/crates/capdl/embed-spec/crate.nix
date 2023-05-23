{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "capdl-embed-spec";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    capdl-types
  ];
  dependencies = {
    capdl-types.features = [ "alloc" "deflate" ];
    inherit (versions) serde_json;
    syn = { version = versions.syn; features = [ "full" ]; };
    inherit (versions) serde;
    inherit (versions) proc-macro2;
    inherit (versions) quote;
    hex = "0.4.3";
  };
}
