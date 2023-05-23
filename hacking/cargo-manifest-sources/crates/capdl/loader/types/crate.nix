{ mk, localCrates, postcardWith, serdeWith }:

mk {
  package.name = "capdl-loader-types";
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
