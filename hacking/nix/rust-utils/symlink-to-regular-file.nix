#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand }:

name: path:

runCommand name {} ''
  cp -L ${path} $out
''
