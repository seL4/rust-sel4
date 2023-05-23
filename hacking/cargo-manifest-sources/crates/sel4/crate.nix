{ mk, localCrates, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4";
  package.license = "MIT";
  nix.local.dependencies = with localCrates; [
    sel4-config
    sel4-sys
  ];
  dependencies = {
    inherit (versions) cfg-if;
  };
  features = {
    default = [ "state" ];
    state = [];
    single-threaded = [];
  };
}
