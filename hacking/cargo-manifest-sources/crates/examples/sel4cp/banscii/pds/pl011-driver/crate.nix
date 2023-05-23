{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "banscii-pl011-driver";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4cp
    banscii-pl011-driver-interface-types
  ];
  dependencies = {
    inherit (versions) tock-registers;
    sel4cp.default-features = false;
    inherit (versions) heapless;
  };
  nix.meta.skip = true;
}
