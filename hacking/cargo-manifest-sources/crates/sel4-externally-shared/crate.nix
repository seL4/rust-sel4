{ mk }:

mk {
  package.name = "sel4-externally-shared";
  package.license = "MIT OR Apache-2.0";
  features = {
    alloc = [];
    unstable = [];
    very_unstable = ["unstable"];
  };
  dev-dependencies = {
    rand = "0.8.3";
  };
  nix.meta.requirements = [ "sel4" ];
}
