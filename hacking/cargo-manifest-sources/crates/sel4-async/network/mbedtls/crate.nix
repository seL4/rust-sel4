{ mk, localCrates, versions, mbedtlsWith, mbedtlsSysAutoWith, mbedtlsPlatformSupportWith }:

mk {
  package.name = "sel4-async-network-mbedtls";
  dependencies = {
    inherit (versions) log;
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
    rand = {
      version = "0.8.5";
      default-features = false;
      features = [
        "small_rng"
      ];
    };
    mbedtls = mbedtlsWith [];
  };
  nix.local.dependencies = with localCrates; [
    sel4-async-network
    mbedtls
  ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true;
}
