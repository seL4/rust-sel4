{ mk, localCrates, versions }:

mk {
  package.name = "sel4-shared-ring-buffer-block-io-types";
  dependencies = {
    inherit (versions) zerocopy;
    num_enum = { version = versions.num_enum; default-features = false; };
  };
  nix.local.dependencies = with localCrates; [
    sel4-shared-ring-buffer
  ];
}
