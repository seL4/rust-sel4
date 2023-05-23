{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "linux" "sel4-config" ];
  package.name = "sel4-config-data";
  nix.local.dependencies = with localCrates; [
    sel4-config-generic-types
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-config-generic-types
    sel4-build-env
  ];
  build-dependencies = {
    sel4-config-generic-types.features = [ "serde" ];
    inherit (versions) serde_json;
  };
  dependencies = {
    sel4-config-generic-types.features = [ "serde" ];
    lazy_static = "1.4.0";
    inherit (versions) serde_json;
  };
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
}
