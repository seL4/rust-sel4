{ mk, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "sel4-generate-target-specs";
  dependencies = {
    inherit (versions) serde_json;
    clap = "3.2.23";
  };
  nix.meta.skip = true;
}
