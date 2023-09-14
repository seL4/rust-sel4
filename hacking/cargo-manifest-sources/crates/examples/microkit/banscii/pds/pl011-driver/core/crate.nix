{ mk, localCrates, versions }:

mk {
  package.name = "banscii-pl011-driver-core";
  dependencies = {
    inherit (versions) tock-registers;
  };
}
