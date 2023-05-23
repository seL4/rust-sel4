{ mk }:

mk {
  package.name = "sel4-build-env";
  nix.meta.requirements = [ "linux" ];
}
