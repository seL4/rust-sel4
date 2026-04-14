#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand }:

name: path:

runCommand name {} ''
  ln -s ${path} $out
''
