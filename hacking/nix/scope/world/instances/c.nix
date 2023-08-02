{ lib, stdenv, hostPlatform
, cmake, perl, python3Packages

, crateUtils, crates
, mkTask, seL4Modifications
, defaultRustTargetInfo

, mkInstance

, canSimulate
}:

let
  libcDir = "${stdenv.cc.libc}/${hostPlatform.config}";

in
mkInstance {
  rootTask = mkTask rec {
    rootCrate = crates.tests-root-task-c;

    release = false;

    layers = [
      crateUtils.defaultIntermediateLayer
      {
        crates = [
          "sel4"
          "sel4-root-task"
        ];
        modifications = seL4Modifications;
      }
    ];

    commonModifications = {
      modifyDerivation = drv: drv.overrideAttrs (self: super: {
        BINDGEN_EXTRA_CLANG_ARGS = [ "-I${libcDir}/include" ];
        nativeBuildInputs = super.nativeBuildInputs ++ [
          cmake
          perl
          python3Packages.jsonschema
          python3Packages.jinja2
        ];
      });
      modifyConfig = old: lib.recursiveUpdate old {
        target.${defaultRustTargetInfo.name} = {
          rustflags = (old.target.${defaultRustTargetInfo.name}.rustflags or []) ++ [
            # TODO
            # NOTE: won't work because cross gcc always uses hard-coded --with-ld

            # "-C" "linker-flavor=gcc"
            # "-C" "link-arg=-nostartfiles"
            # "-C" "default-linker-libraries=on"

            # "-Z" "gcc-ld=lld"
            # (or)
            # "-Z" "unstable-options"
            # "-C" "link-self-contained=+linker"
            # (or)
            # "-Z" "unstable-options"
            # "-C" "linker-flavor=gnu-lld-cc"

            # "-Z" "verbose"
          ];
        };
      };
    };

    lastLayerModifications = crateUtils.composeModifications seL4Modifications (crateUtils.elaborateModifications {
      modifyDerivation = drv: drv.overrideAttrs (self: super: {
        # NIX_LDFLAGS_AFTER = [ "-lnosys" ]; # NOTE: appease CMake's compiler test
        # NIX_DEBUG = 2;
      });
      # extraCargoFlags = [ "--verbose" ];
    });
  };

  extraPlatformArgs = lib.optionalAttrs canSimulate  {
    canAutomateSimply = true;
  };
}
