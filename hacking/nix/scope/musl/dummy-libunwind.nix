#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommandCC }:

runCommandCC "dummy-libunwind" {} ''
  mkdir -p $out/lib
  touch empty.o
  $AR r $out/lib/libunwind.a empty.o
''
