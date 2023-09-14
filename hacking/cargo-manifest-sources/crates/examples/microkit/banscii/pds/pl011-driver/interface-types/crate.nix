{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "banscii-pl011-driver-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
  nix.local.dependencies = with localCrates; [
    # sel4-microkit-message
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
