{ mk, localCrates, versions }:

mk {
  package.name = "microkit-http-server-example-sp804-driver";
  dependencies = {
    sel4-microkit.default-features = false;
  };
  nix.local.dependencies = with localCrates; [
    sel4-microkit
    sel4-microkit-message
    microkit-http-server-example-sp804-driver-core
    microkit-http-server-example-sp804-driver-interface-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
