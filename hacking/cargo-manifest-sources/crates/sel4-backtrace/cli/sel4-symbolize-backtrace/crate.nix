{ mk, localCrates, versions }:

mk {
  package.name = "sel4-symbolize-backtrace";
  dependencies = {
    sel4-backtrace-types.features = [ "full" ];
    addr2line = "0.20.0";
    clap = "3.2.23";
    memmap = "0.7.0";
    hex = "0.4.3";
    inherit (versions) object;
  };
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
