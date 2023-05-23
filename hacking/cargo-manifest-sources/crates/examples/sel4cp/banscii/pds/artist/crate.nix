{ mk, localCrates }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "banscii-artist";
  nix.local.dependencies = with localCrates; [
    sel4cp
    banscii-artist-interface-types
  ];
  dependencies = {
    sel4cp = { default-features = false; features = [ "alloc" ]; };
    rsa = { version = "0.8.1"; default-features = false; features = [ "pem" "sha2" ]; };
  };
  build-dependencies = {
    rsa = "0.8.1";
  };
  nix.meta.skip = true;
}
