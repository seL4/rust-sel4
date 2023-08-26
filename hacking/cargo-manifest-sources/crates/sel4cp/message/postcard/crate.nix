{ mk, localCrates, serdeWith, postcardWith }:

mk {
  package.name = "sel4cp-message-postcard";
  dependencies = {
    serde = serdeWith [];
    postcard = postcardWith [];
  };
  nix.local.dependencies = with localCrates; [
    sel4cp
  ];
}
