{ mk, localCrates, serdeWith }:

mk {
  package.name = "microkit-http-server-example-sp804-driver-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
  nix.local.dependencies = with localCrates; [
    # sel4-microkit-message-types
  ];
}
