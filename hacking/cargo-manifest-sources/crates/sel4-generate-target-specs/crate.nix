{ mk, versions }:

mk {
  package.name = "sel4-generate-target-specs";
  dependencies = {
    inherit (versions) serde_json;
    clap = "3.2.23";
  };
}
