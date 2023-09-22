{ mk, localCrates }:

mk {
  package.name = "microkit-hello";
  nix.local.dependencies = with localCrates; [
    sel4-microkit
  ];
}
