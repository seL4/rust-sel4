{ lib
, runCommand
, capdl-tool
, objectSizes
, serializeCapDLSpec
, crateUtils
, kernelBinary
, seL4ForBoot
, add-payload-to-loader
, loader-expecting-appended-payload
}:

{ seL4Prefix ? seL4ForBoot, app }:

let

in lib.fix (self: runCommand "loader-with-serialization" {

  nativeBuildInputs = [
    add-payload-to-loader
  ];

  passthru = {
    elf = self;
    split = {
      full = loader-expecting-appended-payload.elf;
      min = self;
    };
  };

} ''
  add-payload-to-loader \
    -v \
    --loader ${loader-expecting-appended-payload.elf} \
    --sel4-prefix ${seL4ForBoot} \
    --app ${app} \
    -o $out
'')
