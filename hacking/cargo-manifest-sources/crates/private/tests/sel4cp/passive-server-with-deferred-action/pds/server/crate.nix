{ mk, localCrates }:

mk {
  package.name = "tests-sel4cp-passive-server-with-deferred-action-pd-server";
  nix.local.dependencies = with localCrates; [
    sel4cp
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
