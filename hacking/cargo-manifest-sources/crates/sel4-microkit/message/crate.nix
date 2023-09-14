{ mk, localCrates, serdeWith }:

mk {
  package.name = "sel4-microkit-message";
  nix.local.dependencies = with localCrates; [
    sel4-microkit
    sel4-microkit-message-types
  ];
  dependencies = {
    serde = serdeWith [] // {
      optional = true;
    };
  };
  features = {
    default = [ "postcard" ];
    postcard = [ "dep:serde" "sel4-microkit-message-types/postcard" ];
  };
}
