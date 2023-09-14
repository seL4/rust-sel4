{ mk, localCrates }:

mk {
  package.name = "banscii-assistant";
  nix.local.dependencies = with localCrates; [
    sel4-microkit
    sel4-microkit-message
    sel4-externally-shared
    banscii-assistant-core
    banscii-pl011-driver-interface-types
    banscii-artist-interface-types
  ];
  dependencies = {
    sel4-microkit = { default-features = false; features = [ "alloc" ]; };
    hex = { version = "0.4.3"; default-features = false; features = [ "alloc" ]; };
    sel4-externally-shared.features = [ "unstable" "alloc" ];
  };
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
