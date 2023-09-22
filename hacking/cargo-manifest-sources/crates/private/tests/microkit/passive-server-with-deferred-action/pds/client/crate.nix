{ mk, localCrates }:

mk {
  package.name = "tests-microkit-passive-server-with-deferred-action-pds-client";
  nix.local.dependencies = with localCrates; [
    sel4-microkit
  ];
}
