{ lib, runCommandCC
, sel4-embed-debug-info
}:

elf:

runCommandCC "elf" {
  nativeBuildInputs = [
    sel4-embed-debug-info
  ];
} ''
  $OBJCOPY --only-keep-debug ${elf} dbg.elf
  sel4-embed-debug-info -i ${elf} -d dbg.elf -o $out
''
