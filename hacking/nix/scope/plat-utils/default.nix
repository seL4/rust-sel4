#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ callPackage }:

{
  qemu = callPackage ./qemu {};
  rpi4 = callPackage ./rpi4 {};

  composeInstanceForPlatformAttrs = a: b: {
    attrs = a.attrs // b.attrs;
    links = a.links ++ b.links;
  };
}
