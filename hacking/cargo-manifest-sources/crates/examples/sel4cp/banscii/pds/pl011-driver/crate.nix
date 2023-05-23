{ mk, localCrates, versions }:

mk {
  package.name = "banscii-pl011-driver";
  dependencies = {
    sel4cp.default-features = false;
    inherit (versions) heapless tock-registers;
  };
  nix.local.dependencies = with localCrates; [
    sel4cp
    banscii-pl011-driver-interface-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
