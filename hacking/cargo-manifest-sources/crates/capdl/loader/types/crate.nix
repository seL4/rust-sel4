{ mk, localCrates, postcardWith, serdeWith, coreLicense, meAsAuthor }:

mk {
  package.name = "capdl-loader-types";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    capdl-types
  ];
  dependencies = {
    capdl-types.features = [
      "alloc" "serde"
      "deflate"
    ];
    postcard = postcardWith [ "alloc" ];
    serde = serdeWith [ "alloc" ];
  };
}
