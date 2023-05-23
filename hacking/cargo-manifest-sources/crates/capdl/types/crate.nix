{ mk, localCrates, serdeWith, coreLicense, meAsAuthor, versions }:

mk {
  package.name = "capdl-types";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    capdl-types-derive
    sel4
  ];
  dependencies = {
    serde = serdeWith [ "derive" "alloc" ] // { optional = true; };
    inherit (versions) cfg-if;
    miniz_oxide = { version = "0.6.2"; default-features = false; optional = true; };
    sel4 = { optional = true; default-features = false; };
  };
  features = {
    alloc = [ "miniz_oxide?/with-alloc" ];
    serde = [ "alloc" "dep:serde" ];
    deflate = [ "dep:miniz_oxide" ];
    borrowed-indirect = [];
  };
}
