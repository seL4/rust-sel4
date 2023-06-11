{ mk }:

mk {
  package.name = "sel4-externally-shared";
  package.license = "MIT OR Apache-2.0";
  features = {
    unstable = [];
    alloc = [];
  };
  nix.meta.requirements = [ "sel4" ];
}
