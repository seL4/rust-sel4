{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "capdl-types";
  dependencies = {
    miniz_oxide = { version = "0.6.2"; default-features = false; optional = true; };
    sel4 = { optional = true; default-features = false; };
    serde = serdeWith [ "derive" "alloc" ] // { optional = true; };
    inherit (versions) cfg-if;
  };
  features = {
    alloc = [ "miniz_oxide?/with-alloc" ];
    serde = [ "alloc" "dep:serde" ];
    deflate = [ "dep:miniz_oxide" ];
    borrowed-indirect = [];
  };
  nix.local.dependencies = with localCrates; [
    capdl-types-derive
    sel4
  ];
}
