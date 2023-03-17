{ lib
, runCommand
, capdl-tool
, objectSizes
, serializeCapDLSpec
, crateUtils
, seL4ForUserspace
, capdl-add-spec-to-loader
, capdl-loader-expecting-serialized-spec
}:

{ spec, fill }:

let
  json = serializeCapDLSpec {
    inherit spec;
  };

in lib.fix (self: runCommand "armed-capdl-loader" {

  nativeBuildInputs = [
    capdl-add-spec-to-loader
  ];

  passthru = {
    inherit spec json fill;
    split = {
      full = capdl-loader-expecting-serialized-spec.split.full;
      min = self;
    };
  };

} ''
  capdl-add-spec-to-loader -v -e ${capdl-loader-expecting-serialized-spec.elf} -f ${json} -d ${fill} -o $out
'')
