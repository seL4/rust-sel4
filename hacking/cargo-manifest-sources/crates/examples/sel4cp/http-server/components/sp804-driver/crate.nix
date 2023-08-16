{ mk, localCrates, versions }:

mk {
  package.name = "tests-capdl-http-server-components-sp804-driver";
  dependencies = {
    sel4cp.default-features = false;
  };
  nix.local.dependencies = with localCrates; [
    sel4cp
    tests-capdl-http-server-components-sp804-driver-core
    tests-capdl-http-server-components-sp804-driver-interface-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
