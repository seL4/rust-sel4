{ mk, localCrates, postcardWith, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-loader";
  nix.local.dependencies = with localCrates; [
    sel4-platform-info
    sel4-logging
    sel4-config
    sel4-loader-payload-types
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-rustfmt-helper
    sel4-platform-info
    sel4-config
    sel4-loader-config-types
    sel4-build-env
    sel4-loader-payload-types
    sel4-loader-embed-page-tables
  ];
  dependencies = {
    sel4-loader-payload-types.features = [ "serde" ];
    postcard = postcardWith [];
    heapless = { version = versions.heapless; features = [ "serde" ]; };
    inherit (versions) cfg-if log;
    inherit (versions) tock-registers;
    aarch64-cpu = "9.0.0";
    spin = "0.9.4";
    psci = { version = "0.1.1"; default-features = false; };
  };
  build-dependencies = {
    inherit (versions) quote;
    inherit (versions) object;
    inherit (versions) serde;
    inherit (versions) serde_yaml;
    postcard = postcardWith [ "alloc" ];
    cc = "1.0.76";
    glob = "0.3.0";
  };
}
