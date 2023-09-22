{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "banscii-artist-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
  nix.local.dependencies = with localCrates; [
    # sel4-microkit-message
  ];
}
