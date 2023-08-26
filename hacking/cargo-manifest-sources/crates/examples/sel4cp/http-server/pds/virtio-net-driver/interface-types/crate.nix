{ mk, localCrates, serdeWith }:

mk {
  package.name = "sel4cp-http-server-example-virtio-net-driver-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
  nix.local.dependencies = with localCrates; [
    # sel4cp-message-types
  ];
}
