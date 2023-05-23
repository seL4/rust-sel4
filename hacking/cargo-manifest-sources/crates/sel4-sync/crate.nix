{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-sync";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-immediate-sync-once-cell
  ];
}
