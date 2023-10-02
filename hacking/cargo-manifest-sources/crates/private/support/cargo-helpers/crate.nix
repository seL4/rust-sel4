{ mk }:

mk {
  package.name = "cargo-helpers";
  dependencies = {
    cargo-util = "0.2.3";
    cargo = "0.73.1";
    clap = "3.2.23";
  };
}
