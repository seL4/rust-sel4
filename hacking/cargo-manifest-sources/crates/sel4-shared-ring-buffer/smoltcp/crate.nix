{ mk, localCrates, versions, smoltcpWith }:

mk {
  package.name = "sel4-shared-ring-buffer-smoltcp";
  dependencies = rec {
    inherit (versions) log;
    smoltcp = smoltcpWith [];
    sel4-externally-shared.features = [ "unstable" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-externally-shared
    sel4-shared-ring-buffer
    sel4-bounce-buffer-allocator
  ];
}
