{ mk, versions }:

mk {
  package.name = "tests-capdl-http-server-components-sp804-driver-interface-types";
  dependencies = {
    num_enum = { version = versions.num_enum; default-features = false; };
    inherit (versions) zerocopy;
  };
}
