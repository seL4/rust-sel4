{ mk, localCrates }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "sel4-bitfield-parser-test";
  nix.local.dependencies = with localCrates; [
    sel4-bitfield-parser
  ];
  dependencies = {
    clap = "3.2.23";
    glob = "0.3.0";
  };
}
