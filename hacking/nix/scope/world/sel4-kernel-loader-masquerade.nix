#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, buildCratesInLayers, buildSysroot, crateUtils
, crates
, mkBuiltinRustTargetTriple, bareMetalBuiltinRustTargetTripleSoftFloat
}:

let
  # NOTE
  # Soft-float, because the Linux boot protocol does not guarantee that FP/SIMD access is
  # enabled on entry, and LLVM uses SIMD registers even for integer code.
  targetTriple = mkBuiltinRustTargetTriple bareMetalBuiltinRustTargetTripleSoftFloat;

  rootCrate = crates.sel4-kernel-loader-masquerade;

  profile = "release";

  pieConfig = {
    target.${targetTriple.name}.rustflags = [
      "-C" "relocation-model=pie"
    ];
  };

  sysroot = buildSysroot {
    inherit targetTriple;
    inherit profile;
    alloc = false;
    extraConfig = pieConfig;
  };

in
buildCratesInLayers {

  inherit rootCrate;
  inherit targetTriple;
  inherit profile;

  commonModifications = {
    modifyConfig = lib.flip crateUtils.combineConfig (crateUtils.clobber [
      pieConfig
      {
        target.${targetTriple.name}.rustflags = [
          "--sysroot" sysroot
        ];
      }
    ]);
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      dontStrip = true;
      dontPatchELF = true;
    });
  };

  lastLayerModifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      passthru = (super.passthru or {}) // {
        elf = "${self.finalPackage}/bin/${rootCrate.name}";
      };
    });
  };

}
