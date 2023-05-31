{ lib
, runCommand
, capdl-tool
, objectSizes
, serializeCapDLSpec
, crateUtils
, kernelBinary
, seL4ForBoot
, sel4-loader-add-payload
, sel4-loader
}:

{ seL4Prefix ? seL4ForBoot, app }:

let

in lib.fix (self: runCommand "sel4-loader-with-payload" {

  nativeBuildInputs = [
    sel4-loader-add-payload
  ];

  passthru = {
    elf = self;
    split = {
      full = sel4-loader.elf;
      min = self;
    };
  };

} ''
  sel4-loader-add-payload \
    -v \
    --loader ${sel4-loader.elf} \
    --sel4-prefix ${seL4ForBoot} \
    --app ${app} \
    -o $out
'')
