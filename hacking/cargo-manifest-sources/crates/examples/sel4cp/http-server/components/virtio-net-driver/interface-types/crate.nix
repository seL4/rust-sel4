{ mk, versions }:

mk {
  package.name = "sel4cp-http-server-example-virtio-net-driver-interface-types";
  dependencies = {
    num_enum = { version = versions.num_enum; default-features = false; };
    inherit (versions) zerocopy;
  };
}
