{ lib, runCommand
, sel4-inject-phdrs
}:

elf:

runCommand "elf" {
  nativeBuildInputs = [
    sel4-inject-phdrs
  ];
} ''
  sel4-inject-phdrs ${elf} -o $out
''
