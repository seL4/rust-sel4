{ mk, localCrates, versions }:

mk {
  package.name = "sel4cp-message";
  dependencies = {
    inherit (versions) cfg-if zerocopy;
    num_enum = { version = versions.num_enum; default-features = false; };
  };
  nix.local.dependencies = with localCrates; [
    sel4cp
  ];
}
