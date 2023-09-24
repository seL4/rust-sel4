{ mk, localCrates, serdeWith }:

mk {
  package.name = "microkit-http-server-example-virtio-blk-driver-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
  nix.local.dependencies = with localCrates; [
    # sel4-microkit-message-types
  ];
}
