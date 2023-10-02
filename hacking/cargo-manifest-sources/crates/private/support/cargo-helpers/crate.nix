{ mk, versions }:

mk {
  package.name = "cargo-helpers";
  dependencies = {
    inherit (versions) clap;
    cargo-util = "0.2.3";
    cargo = "0.73.1";
  };
}
