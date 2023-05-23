{ mk, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "sel4-generate-target-specs";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    inherit (versions) serde_json;
    clap = "3.2.23";
  };
  nix.meta.skip = true;
}
