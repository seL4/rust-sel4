{ mk, versions }:

mk {
  package.name = "microkit-http-server-example-sp804-driver-core";
  dependencies = rec {
    inherit (versions) log;
    tock-registers = "0.8.1";
  };
}
