{ mk, localCrates, versions }:

mk {
  package.name = "sel4-platform-info";
  nix.local.dependencies = with localCrates; [
    sel4-platform-info-types
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-build-env
  ];
  build-dependencies = {
    inherit (versions) proc-macro2;
    inherit (versions) quote;
    serde = { version = versions.serde; features = [ "derive" ]; };
    inherit (versions) serde_yaml;
  };
  nix.meta.requirements = [ "sel4" ];
}
