{ lib
, runCommand
, capdl-tool
, objectSizes
, serializeCapDLSpec
, crateUtils
, capdl-initializer-add-spec
, capdl-initializer
}:

{ spec, fill }:

let
  json = serializeCapDLSpec {
    inherit spec;
  };

in lib.fix (self: runCommand "capdl-initializer-with-spec" {

  nativeBuildInputs = [
    capdl-initializer-add-spec
  ];

  passthru = {
    inherit spec json fill;
    elf = self;
    split = {
      full = capdl-initializer.elf;
      min = self;
    };
  };

} ''
  capdl-initializer-add-spec \
    -v \
    -e ${capdl-initializer.elf} \
    -f ${json} \
    -d ${fill} \
    --object-names-level 2 \
    -o $out
'')
