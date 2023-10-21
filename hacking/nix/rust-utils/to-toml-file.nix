#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand, remarshal }:

name: expr:

runCommand name {
  nativeBuildInputs = [
    remarshal
  ];
  json = builtins.toJSON expr;
  passAsFile = [ "json" ];
} ''
  remarshal -if json -of toml -i $jsonPath -o $out
''
