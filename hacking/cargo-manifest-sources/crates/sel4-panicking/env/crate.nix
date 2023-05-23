{ mk }:

mk {
  package.name = "sel4-panicking-env";
  nix.meta.requirements = [ "sel4" ];
}
