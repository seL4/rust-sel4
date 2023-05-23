{ mk }:

mk {
  package.name = "sel4-externally-shared";
  package.license = "MIT";
  features = {
    unstable = [];
    alloc = [];
  };
  nix.meta.requirements = [ "sel4" ];
}
