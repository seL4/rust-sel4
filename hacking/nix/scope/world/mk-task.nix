{ lib, buildPackages
, buildCrateInLayersHere, buildSysroot, crateUtils
, crates
, defaultRustTargetInfo
, seL4RustEnvVars
} @ scopeArgs:

{ commonModifications ? {}
, lastLayerModifications ? {}

, extraProfile ? {}
, replaceSysroot ? null

, rustTargetInfo ? defaultRustTargetInfo
, release ? true
, ...
} @ args:

let
  profile = if release then "release" else "dev";

  profiles = crateUtils.clobber [
    {
      profile.release = {
        lto = true;
      };
    }
    {
      profile.${profile} = {
        codegen-units = 1;
        incremental = false;
      } // extraProfile;
    }
  ];

  sysroot = (if replaceSysroot != null then replaceSysroot else buildSysroot) {
    inherit release rustTargetInfo;
    extraManifest = profiles;
  };

  theseCommonModifications = crateUtils.elaborateModifications {
    modifyManifest = lib.flip lib.recursiveUpdate profiles;
    modifyConfig = lib.flip lib.recursiveUpdate {
      target.${rustTargetInfo.name}.rustflags = [
        "--sysroot" sysroot
      ];
    };
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      # TODO
      # hardeningDisable = [ "all" ];

      LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";

      dontStrip = true;
      dontPatchELF = true;
    });
  };

  theseLastLayerModifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars // {
      passthru = (super.passthru or {}) // rec {
        elf = "${self.finalPackage}/bin/${args.rootCrate.name}.elf";
        # HACK
        split = {
          min = elf;
          full = elf;
        };
      };
    });
  };

  prunedArgs = builtins.removeAttrs args [
    "extraProfile"
    "replaceSysroot"
  ];

in

buildCrateInLayersHere (prunedArgs // {

  commonModifications = crateUtils.composeModifications
    (crateUtils.elaborateModifications commonModifications)
    theseCommonModifications
  ;

  lastLayerModifications = crateUtils.composeModifications
    (crateUtils.elaborateModifications lastLayerModifications)
    theseLastLayerModifications
  ;
})
