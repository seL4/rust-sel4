{ mk, localCrates }:

mk {
  package.name = "tests-microkit-passive-server-with-deferred-action-pds-client";
  nix.local.dependencies = with localCrates; [
    sel4-microkit
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
