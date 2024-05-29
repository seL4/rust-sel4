#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, buildPackages, writeText
, buildCrateInLayers, buildSysroot, crateUtils
, crates, bareMetalRustTargetTriple
, libclangPath
, seL4RustEnvVars, seL4ForBoot, seL4ForUserspace
, kernelLoaderConfig
}:

let
  targetTriple = bareMetalRustTargetTriple;

  rootCrate = crates.sel4-kernel-loader;

  profile = "release";
  # profile = "dev";

  profiles = crateUtils.clobber [
    {
      profile.release = {
        lto = true;
      };
    }
    {
      profile.${profile} = {
        # overflow-checks = true; # TODO
        codegen-units = 1;
        incremental = false;
        # debug = 2;
      };
    }
  ];

  sysroot = buildSysroot {
    inherit targetTriple;
    inherit profile;
    extraManifest = profiles;
    alloc = false;
  };

in
buildCrateInLayers {

  inherit rootCrate;
  inherit targetTriple;
  inherit profile;

  features = [];

  commonModifications = {
    modifyManifest = lib.flip lib.recursiveUpdate profiles;
    modifyConfig = lib.flip lib.recursiveUpdate {
      target.${targetTriple.name}.rustflags = [
        "--sysroot" sysroot
      ];
    };
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      LIBCLANG_PATH = libclangPath;

      dontStrip = true;
      dontPatchELF = true;
    });
  };

  lastLayerModifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars //{
      SEL4_KERNEL_LOADER_CONFIG = writeText "loader-config.json" (builtins.toJSON kernelLoaderConfig);

      # SEL4_KERNEL = "${seL4ForBoot}/bin/kernel.elf";

      passthru = (super.passthru or {}) // {
        elf = "${self.finalPackage}/bin/${rootCrate.name}";
      };
    });
  };

}
