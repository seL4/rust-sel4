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

{ kernel ? kernelBinary, app }:

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
    --kernel ${kernel} \
    --app ${app} \
    --dtb ${seL4ForBoot}/support/kernel.dtb \
    --platform-info ${seL4ForBoot}/support/platform_gen.yaml \
    -o $out

  # cp ${loader-expecting-appended-payload.elf} $out
'')
