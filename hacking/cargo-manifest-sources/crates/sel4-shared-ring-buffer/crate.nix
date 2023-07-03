{ mk, localCrates, versions }:

mk {
  package.name = "sel4-shared-ring-buffer";
  dependencies = rec {
    inherit (versions) log zerocopy;
  };
  nix.local.dependencies = with localCrates; [
    sel4-externally-shared
  ];
}
