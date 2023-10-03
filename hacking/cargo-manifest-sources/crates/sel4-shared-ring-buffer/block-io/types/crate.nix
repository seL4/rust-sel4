{ mk, localCrates, versions, zerocopyWith }:

mk {
  package.name = "sel4-shared-ring-buffer-block-io-types";
  dependencies = {
    num_enum = { version = versions.num_enum; default-features = false; };
    zerocopy = zerocopyWith [ "derive" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-shared-ring-buffer
  ];
}
