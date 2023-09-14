{ mk, versions, serdeWith, postcardWith }:

mk {
  package.name = "sel4-microkit-message-types";
  dependencies = {
    inherit (versions) zerocopy;
    num_enum = { version = versions.num_enum; default-features = false; };
    serde = serdeWith [] // {
      optional = true;
    };
    postcard = postcardWith [] // {
      optional = true;
    };
  };
  features = {
    default = [ "postcard" ];
    postcard = [ "dep:postcard" "dep:serde" ];
  };
}
