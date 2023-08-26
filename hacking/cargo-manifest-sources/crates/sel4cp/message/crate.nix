{ mk, localCrates, serdeWith }:

mk {
  package.name = "sel4cp-message";
  nix.local.dependencies = with localCrates; [
    sel4cp
    sel4cp-message-types
  ];
  dependencies = {
    serde = serdeWith [] // {
      optional = true;
    };
  };
  features = {
    default = [ "postcard" ];
    postcard = [ "dep:serde" "sel4cp-message-types/postcard" ];
  };
}
