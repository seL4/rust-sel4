#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, callPackage

, worldConfig
, seL4Config

, maybe
, canSimulate
}:

let
  inherit (worldConfig) isMicrokit;

in {
  serial = maybe
    (isMicrokit && seL4Config.PLAT == "qemu-arm-virt")
    (callPackage ./serial {
    });
}
