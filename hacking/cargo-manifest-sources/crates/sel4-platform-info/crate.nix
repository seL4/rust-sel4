{ mk, localCrates, versions }:

mk {
  package.name = "sel4-platform-info";
  build-dependencies = {
    serde = { version = versions.serde; features = [ "derive" ]; };
    inherit (versions) proc-macro2 quote serde_yaml;
  };
  nix.local.dependencies = with localCrates; [
    sel4-platform-info-types
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-build-env
    sel4-config
  ];
}
