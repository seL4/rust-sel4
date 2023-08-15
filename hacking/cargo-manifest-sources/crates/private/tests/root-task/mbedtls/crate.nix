{ mk, localCrates, versions, mbedtlsWith, mbedtlsPlatformSupportWith, mbedtlsSysAutoWith }:

mk {
  package.name = "tests-root-task-mbedtls";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task
    sel4-logging
    sel4-newlib
    # mbedtls
    # mbedtls-platform-support
    # mbedtls-sys-auto
  ];
  dependencies = {
    inherit (versions) log;
    mbedtls = mbedtlsWith [
      "debug"
    ];
    mbedtls-platform-support = mbedtlsPlatformSupportWith [
    ];
    mbedtls-sys-auto = mbedtlsSysAutoWith [
    ];
    sel4-newlib = {
      features = [
        "nosys"
        "all-symbols"
        "sel4-panicking-env"
      ];
    };
  };
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
}
