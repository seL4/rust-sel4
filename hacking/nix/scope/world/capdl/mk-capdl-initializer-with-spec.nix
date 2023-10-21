#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, runCommand
, capdl-tool
, objectSizes
, serializeCapDLSpec
, crateUtils
, sel4-capdl-initializer-add-spec
, sel4-capdl-initializer
}:

{ spec, fill }:

let
  json = serializeCapDLSpec {
    inherit spec;
  };

in lib.fix (self: runCommand "sel4-capdl-initializer-with-spec" {

  nativeBuildInputs = [
    sel4-capdl-initializer-add-spec
  ];

  passthru = {
    inherit spec json fill;
    elf = self;
    split = {
      full = sel4-capdl-initializer.elf;
      min = self;
    };
  };

} ''
  sel4-capdl-initializer-add-spec \
    -v \
    -e ${sel4-capdl-initializer.elf} \
    -f ${json} \
    -d ${fill} \
    --object-names-level 2 \
    --embed-frames \
    -o $out
'')
