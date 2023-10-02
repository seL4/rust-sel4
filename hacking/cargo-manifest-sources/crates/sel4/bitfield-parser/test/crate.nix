{ mk, localCrates, versions }:

mk {
  package.name = "sel4-bitfield-parser-test";
  dependencies = {
    inherit (versions) clap;
    glob = "0.3.0";
  };
  nix.local.dependencies = with localCrates; [
    sel4-bitfield-parser
  ];
}
