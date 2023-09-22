{ mk, localCrates }:

mk {
  package.name = "sel4-bitfield-parser-test";
  dependencies = {
    clap = "3.2.23";
    glob = "0.3.0";
  };
  nix.local.dependencies = with localCrates; [
    sel4-bitfield-parser
  ];
}
