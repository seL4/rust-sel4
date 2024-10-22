#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand
, muslForSeL4Raw
}:

runCommand "musl" {} ''
  mkdir -p $out/lib
  ln -s ${muslForSeL4Raw}/include $out
  ln -s ${muslForSeL4Raw}/lib/libc.a $out/lib
''
