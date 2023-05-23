{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-build-env";
}
