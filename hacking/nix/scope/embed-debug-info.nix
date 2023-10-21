#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, runCommandCC
, sel4-backtrace-embedded-debug-cli
}:

elf:

runCommandCC "elf" {
  nativeBuildInputs = [
    sel4-backtrace-embedded-debug-cli
  ];
} ''
  $OBJCOPY --only-keep-debug ${elf} dbg.elf
  sel4-embed-debug-info -i ${elf} -d dbg.elf -o $out
''
