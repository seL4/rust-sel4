{ mk, localCrates, versions }:

mk {
  package.name = "sel4-config-data";
  dependencies = {
    sel4-config-generic-types.features = [ "serde" ];
    lazy_static = "1.4.0";
    inherit (versions) serde_json;
  };
  build-dependencies = {
    sel4-config-generic-types.features = [ "serde" ];
    inherit (versions) serde_json;
  };
  nix.local.dependencies = with localCrates; [
    sel4-config-generic-types
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-config-generic-types
    sel4-build-env
  ];
}
