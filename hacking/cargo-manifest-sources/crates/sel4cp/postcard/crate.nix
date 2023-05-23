{ mk, localCrates, serdeWith, postcardWith }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4cp-postcard";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4cp
  ];
  dependencies = {
    serde = serdeWith [];
    postcard = postcardWith [];
  };
  nix.meta.skip = true;
}
