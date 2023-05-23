{ mk, localCrates, serdeWith, postcardWith }:

mk {
  package.name = "sel4cp-postcard";
  dependencies = {
    serde = serdeWith [];
    postcard = postcardWith [];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4cp
  ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
