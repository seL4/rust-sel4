{ mk, localCrates, versions }:

mk {
  package.name = "banscii-pl011-driver-interface-types";
  dependencies = {
    num_enum = { version = versions.num_enum; default-features = false; };
    inherit (versions) zerocopy;
  };
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
