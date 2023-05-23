{ mk }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-immediate-sync-once-cell";
}
