{ mk, localCrates, versions, volatileSource }:

mk {
  package.name = "sel4-externally-shared";
  nix.local.dependencies = with localCrates; [
    # volatile
  ];
  dependencies = {
    inherit (versions) zerocopy;
    volatile = volatileSource;
  };
  features = {
    "unstable" = [ "volatile/unstable" ];
    "very_unstable" = [ "volatile/very_unstable" ];
  };
}
