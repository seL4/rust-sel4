{ mk, coreLicense, meAsAuthor }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "cargo-helpers";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    cargo-util = "0.2.3";
    cargo = "0.70.1";
    clap = "3.2.23";
  };
}
