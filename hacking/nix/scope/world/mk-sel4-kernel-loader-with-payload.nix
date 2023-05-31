{ lib
, runCommand
, capdl-tool
, objectSizes
, serializeCapDLSpec
, crateUtils
, kernelBinary
, seL4ForBoot
, sel4-kernel-loader-add-payload
, sel4-kernel-loader
}:

{ seL4Prefix ? seL4ForBoot, app }:

let

in lib.fix (self: runCommand "sel4-kernel-loader-with-payload" {

  nativeBuildInputs = [
    sel4-kernel-loader-add-payload
  ];

  passthru = {
    elf = self;
    split = {
      full = sel4-kernel-loader.elf;
      min = self;
    };
  };

} ''
  sel4-kernel-loader-add-payload \
    -v \
    --loader ${sel4-kernel-loader.elf} \
    --sel4-prefix ${seL4ForBoot} \
    --app ${app} \
    -o $out
'')
