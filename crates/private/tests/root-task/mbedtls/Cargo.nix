#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, mbedtlsWith, mbedtlsPlatformSupportWith, mbedtlsSysAutoWith }:

mk {
  package.name = "tests-root-task-mbedtls";
  dependencies = {
    inherit (versions) log;
    mbedtls = mbedtlsWith [
      "debug"
    ];
    mbedtls-platform-support = mbedtlsPlatformSupportWith [
    ];
    mbedtls-sys-auto = mbedtlsSysAutoWith [
    ];
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-logging
      # mbedtls
      # mbedtls-platform-support
      # mbedtls-sys-auto
    ;
    sel4-newlib = localCrates.sel4-newlib // {
      features = [
        "nosys"
        "all-symbols"
        "sel4-panicking-env"
      ];
    };
  };
}
