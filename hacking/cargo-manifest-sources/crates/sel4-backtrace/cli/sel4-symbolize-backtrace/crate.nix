{ mk, localCrates, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "sel4-symbolize-backtrace";
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-types
  ];
  dependencies = {
    sel4-backtrace-types.features = [ "full" ];
    addr2line = "0.20.0";
    clap = "3.2.23";
    memmap = "0.7.0";
    inherit (versions) object;
    hex = "0.4.3";
  };
}
