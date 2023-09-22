{ mk, versions }:

mk {
  package.name = "sel4-logging";
  dependencies = {
    inherit (versions) log;
  };
}
