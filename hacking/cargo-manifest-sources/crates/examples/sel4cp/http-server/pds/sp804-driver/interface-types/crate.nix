{ mk, localCrates, serdeWith }:

mk {
  package.name = "sel4cp-http-server-example-sp804-driver-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
  nix.local.dependencies = with localCrates; [
    # sel4cp-message-types
  ];
}
