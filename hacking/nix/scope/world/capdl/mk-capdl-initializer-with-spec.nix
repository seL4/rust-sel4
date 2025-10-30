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

{ cdl
, fill
, embedFrames ? true
, deflate ? true
, alloc ? true
}:

let
  json = serializeCapDLSpec {
    inherit cdl;
  };

  initializer = sel4-capdl-initializer.override {
    inherit deflate alloc;
  };

in lib.fix (self: runCommand "sel4-capdl-initializer-with-spec" {

  nativeBuildInputs = [
    sel4-capdl-initializer-add-spec
  ];

  passthru = {
    inherit cdl json fill;
    elf = self;
    split = {
      full = initializer.elf;
      min = self;
    };
  };

} ''
  sel4-capdl-initializer-add-spec \
    -v \
    -e ${initializer.elf} \
    -f ${json} \
    -d ${fill} \
    --object-names-level 2 \
    ${lib.optionalString (!embedFrames) "--no-embed-frames"} \
    ${lib.optionalString (!deflate) "--no-deflate"} \
    -o $out
'')
