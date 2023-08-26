{ mk, localCrates, versions }:

mk {
  package.name = "sel4cp-http-server-example-sp804-driver";
  dependencies = {
    sel4cp.default-features = false;
  };
  nix.local.dependencies = with localCrates; [
    sel4cp
    sel4cp-message
    sel4cp-http-server-example-sp804-driver-core
    sel4cp-http-server-example-sp804-driver-interface-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
