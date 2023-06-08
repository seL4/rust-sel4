{ callPackage }:

{
  qemu = callPackage ./qemu {};
  rpi4 = callPackage ./rpi4 {};
}
