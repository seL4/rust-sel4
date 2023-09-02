{ mk, localCrates, versions, postcardWith }:

mk {
  package.name = "sel4-kernel-loader";
  dependencies = {
    sel4-kernel-loader-payload-types.features = [ "serde" ];
    postcard = postcardWith [];
    heapless = { version = versions.heapless; features = [ "serde" ]; };
    aarch64-cpu = "9.0.0";
    spin = "0.9.4";
    psci = { version = "0.1.1"; default-features = false; };
    inherit (versions) tock-registers cfg-if log;
  };
  target."cfg(any(target_arch = \"riscv32\", target_arch = \"riscv64\"))".dependencies = {
    sbi = "0.2.0";
    riscv = "0.10.0";
  };
  build-dependencies = {
    postcard = postcardWith [ "alloc" ];
    cc = "1.0.76";
    glob = "0.3.0";
    inherit (versions)
      quote
      object
      serde
      serde_yaml
    ;
  };
  nix.local.dependencies = with localCrates; [
    sel4-platform-info
    sel4-logging
    sel4-config
    sel4-kernel-loader-payload-types
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-rustfmt-helper
    sel4-platform-info
    sel4-config
    sel4-kernel-loader-config-types
    sel4-build-env
    sel4-kernel-loader-payload-types
    sel4-kernel-loader-embed-page-tables
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
