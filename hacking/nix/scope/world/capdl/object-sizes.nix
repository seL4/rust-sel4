#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommandCC, sources, libsel4 }:

runCommandCC "object_sizes.yaml" {
  buildInputs = [ libsel4 ];
} ''
  $CC -E -P - < ${sources.objectSizes}/object_sizes.yaml > $out
''
