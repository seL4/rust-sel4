{ mk, localCrates, versions }:

mk {
  package.name = "sel4-backtrace-cli";
  dependencies = {
    sel4-backtrace-types.features = [ "full" ];
    memmap = "0.7.0";
    hex = "0.4.3";
    inherit (versions) object addr2line clap;
  };
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-types
  ];
}
