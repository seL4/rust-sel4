{ mk, localCrates, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-logging";
  dependencies = {
    inherit (versions) log;
  };
}
