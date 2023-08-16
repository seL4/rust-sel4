{ mk, versions }:

mk {
  package.name = "tests-capdl-http-server-components-sp804-driver-core";
  dependencies = rec {
    inherit (versions) log;
    tock-registers = "0.8.1";
  };
}
