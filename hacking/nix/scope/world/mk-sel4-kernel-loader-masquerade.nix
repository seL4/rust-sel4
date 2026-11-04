#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, runCommand
, sel4-kernel-loader-masquerade-add-payload
, sel4-kernel-loader-masquerade
}:

{ loaderELF }:

lib.fix (self: runCommand "sel4-kernel-loader-masquerade.img" {

  nativeBuildInputs = [
    sel4-kernel-loader-masquerade-add-payload
  ];

  passthru = {
    elf = self;
    split = {
      full = sel4-kernel-loader-masquerade.elf;
      min = self;
    };
  };

} ''
  sel4-kernel-loader-masquerade-add-payload \
    --loader ${sel4-kernel-loader-masquerade.elf} \
    --payload ${loaderELF} \
    -o $out
'')
