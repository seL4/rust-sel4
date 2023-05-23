{ mk }:

mk {
  package.name = "sel4-immediate-sync-once-cell";
  nix.meta.requirements = [ "sel4" ];
}
