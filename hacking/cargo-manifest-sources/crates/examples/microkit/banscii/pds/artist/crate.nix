{ mk, localCrates }:

mk {
  package.name = "banscii-artist";
  dependencies = {
    sel4-microkit = { default-features = false; features = [ "alloc" ]; };
    rsa = { version = "0.8.1"; default-features = false; features = [ "pem" "sha2" ]; };
    sel4-externally-shared.features = [ "unstable" "alloc" ];
  };
  build-dependencies = {
    rsa = "0.8.1";
  };
  nix.local.dependencies = with localCrates; [
    sel4-microkit
    sel4-microkit-message
    sel4-externally-shared
    banscii-artist-interface-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
