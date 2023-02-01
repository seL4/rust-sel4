{ lib, buildPackages, writeText
, buildCrateInLayersHere, buildSysroot, crateUtils
, crates, bareMetalRustTargetInfo
, seL4ForUserspace, seL4ForBoot
, loaderConfig
}:

{ appELF }:

let
  rustTargetInfo = bareMetalRustTargetInfo;
  rustTargetName = rustTargetInfo.name;
  rustTargetPath = rustTargetInfo.path;

  release = false;

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
      };
    }
  ];

  sysroot = buildSysroot {
    inherit release rustTargetInfo;
    extraManifest = profiles;
  };

  rootCrate = crates.loader;

  intermediateModifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      SEL4_PREFIX = seL4ForUserspace;
      SEL4_LOADER_CONFIG = writeText "loader-config.json" (builtins.toJSON loaderConfig);
    });
  };

in
buildCrateInLayersHere {

  layers = [
    crateUtils.defaultIntermediateLayer
    {
      crates = [ "loader-core" ];
      modifications = intermediateModifications;
    }
  ];

  inherit release;
  inherit rootCrate;

  rustTargetInfo = bareMetalRustTargetInfo;

  features = [];

  commonModifications = {
    modifyManifest = lib.flip lib.recursiveUpdate profiles;
    modifyConfig = lib.flip lib.recursiveUpdate {
      target.${rustTargetName}.rustflags = [
        "--sysroot" sysroot
      ];
    };
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";

      dontStrip = true;
      dontPatchELF = true;
    });
  };

  lastLayerModifications = crateUtils.composeModifications intermediateModifications (crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      SEL4_KERNEL = "${seL4ForBoot}/bin/kernel.elf";
      SEL4_DTB = "${seL4ForBoot}/support/kernel.dtb";
      SEL4_APP = appELF;

      passthru = (super.passthru or {}) // {
        elf = "${self.finalPackage}/bin/${rootCrate.name}";
      };
    });
  });

}
