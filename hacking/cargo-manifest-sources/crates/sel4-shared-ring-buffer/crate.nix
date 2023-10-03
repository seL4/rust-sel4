{ mk, localCrates, versions, zerocopyWith }:

mk {
  package.name = "sel4-shared-ring-buffer";
  dependencies = rec {
    inherit (versions) log;
    zerocopy = zerocopyWith [ "derive" ];
    sel4-externally-shared.features = [ "unstable" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-externally-shared
  ];
}
