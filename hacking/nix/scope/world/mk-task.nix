#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, buildPackages
, runCommand, runCommandCC
, buildCratesInLayers, buildSysroot, crateUtils
, defaultRustEnvironment
, defaultRustTargetTriple
, libclangPath
, seL4RustEnvVars
} @ scopeArgs:

let
  getELFDefault = drv: "${drv}/bin/${drv.rootCrate.name}.elf";
  getELFDefaultForTest = drv: runCommand "test.elf" {} ''
    ln -sT ${drv}/bin/*.elf $out
  '';
in

{ rustEnvironment ? defaultRustEnvironment
, targetTriple ? defaultRustTargetTriple

, commonModifications ? {}
, lastLayerModifications ? {}

, extraProfile ? {}
, replaceSysroot ? null
, getELF ? if test then getELFDefaultForTest else getELFDefault

, release ? true
, profile ? if release then "release" else (if test then "test" else "dev")

, test ? false

, ...
} @ args:

let
  profiles = crateUtils.clobber [
    {
      profile.release = {
        lto = true;
      };
    }
    {
      profile.${profile} = crateUtils.clobber [
        {
          codegen-units = 1;
          incremental = false;
          build-override = {
            opt-level = 0;
            debug = false;
            strip = true;
          };
        }
        extraProfile
      ];
    }
  ];

  sysroot = (if replaceSysroot != null then replaceSysroot else buildSysroot) {
    inherit rustEnvironment;
    inherit targetTriple;
    inherit profile;
    extraManifest = profiles;
  };

  theseCommonModifications = crateUtils.elaborateModifications {
    modifyManifest = lib.flip lib.recursiveUpdate profiles;
    modifyConfig = lib.flip lib.recursiveUpdate {
      target.${targetTriple.name}.rustflags = [
        "--sysroot" sysroot
      ];
    };
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      # TODO
      # hardeningDisable = [ "all" ];

      LIBCLANG_PATH = libclangPath;

      dontStrip = true;
      dontPatchELF = true;
    });
  };

  theseLastLayerModifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars // {
      passthru = (super.passthru or {}) // {
        elf = getELF self.finalPackage;
        split = {
          full = self.finalPackage.elf;
          min = runCommandCC "stripped.elf" {} ''
            $STRIP -s ${self.finalPackage.elf} -o $out
          '';
        };
      };
    });
  };

  prunedArgs = builtins.removeAttrs args [
    "extraProfile"
    "replaceSysroot"
    "getELF"
  ];

in

buildCratesInLayers (prunedArgs // {
  commonModifications = crateUtils.composeModifications
    (crateUtils.elaborateModifications commonModifications)
    theseCommonModifications
  ;

  lastLayerModifications = crateUtils.composeModifications
    (crateUtils.elaborateModifications lastLayerModifications)
    theseLastLayerModifications
  ;
})
