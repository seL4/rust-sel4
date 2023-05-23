{ mk, localCrates }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "banscii-assistant";
  nix.local.dependencies = with localCrates; [
    sel4cp
    banscii-assistant-core
    banscii-pl011-driver-interface-types
    banscii-artist-interface-types
  ];
  dependencies = {
    sel4cp = { default-features = false; features = [ "alloc" ]; };
    hex = { version = "0.4.3"; default-features = false; features = [ "alloc" ]; };
  };
  nix.meta.skip = true;
}
