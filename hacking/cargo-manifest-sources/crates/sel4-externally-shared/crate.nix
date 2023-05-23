{ mk }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-externally-shared";
  package.license = "MIT";
  features = {
    unstable = [];
    alloc = [];
  };
}
