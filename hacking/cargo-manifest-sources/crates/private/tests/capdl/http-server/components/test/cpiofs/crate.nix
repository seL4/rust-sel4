{ mk, versions }:

mk {
  package.name = "tests-capdl-http-server-components-test-cpiofs";
  dependencies = rec {
    inherit (versions) log zerocopy;
    hex = { version = "0.4.3"; default-features = false; };
  };
}
