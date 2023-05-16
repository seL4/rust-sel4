{ lib
, runCommand
, capdl-tool
, objectSizes
, serializeCapDLSpec
, crateUtils
, kernelBinary
, seL4ForBoot
, sel4-loader-add-payload
, loader-expecting-appended-payload
}:

{ seL4Prefix ? seL4ForBoot, app }:

let

in lib.fix (self: runCommand "loader-with-serialization" {

  nativeBuildInputs = [
    sel4-loader-add-payload
  ];

  passthru = {
    elf = self;
    split = {
      full = loader-expecting-appended-payload.elf;
      min = self;
    };
  };

} ''
  sel4-loader-add-payload \
    -v \
    --loader ${loader-expecting-appended-payload.elf} \
    --sel4-prefix ${seL4ForBoot} \
    --app ${app} \
    -o $out
'')
