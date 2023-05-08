{ lib
, runCommand
, capdl-tool
, objectSizes
, serializeCapDLSpec
, crateUtils
, capdl-loader-add-spec
, capdl-loader
}:

{ spec, fill }:

let
  json = serializeCapDLSpec {
    inherit spec;
  };

in lib.fix (self: runCommand "armed-capdl-loader" {

  nativeBuildInputs = [
    capdl-loader-add-spec
  ];

  passthru = {
    inherit spec json fill;
    elf = self;
    split = {
      full = capdl-loader.elf;
      min = self;
    };
  };

} ''
  capdl-loader-add-spec \
    -v \
    -e ${capdl-loader.elf} \
    -f ${json} \
    -d ${fill} \
    --object-names-level 2 \
    -o $out
'')
