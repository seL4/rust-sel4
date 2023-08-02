{ lib, stdenv, hostPlatform
, cmake

, crates
, mkTask
, defaultRustTargetInfo
, stdenvWithLibc

, mkInstance
}:

let
  libcDir = "${stdenvWithLibc.cc.libc}/${hostPlatform.config}";

in
mkInstance {
  rootTask = mkTask rec {
    stdenv = stdenvWithLibc;

    rootCrate = crates.tests-root-task-c;

    release = false;

    commonModifications = {
      modifyDerivation = drv: drv.overrideAttrs (self: super: {
        NIX_LDFLAGS_AFTER = [ "-lnosys" ]; # appease CMake's compiler test
        BINDGEN_EXTRA_CLANG_ARGS = [ "-I${libcDir}/include" ];
        nativeBuildInputs = super.nativeBuildInputs ++ [
          cmake
        ];
        # NIX_DEBUG = 2;
      });
      # extraCargoFlags = [ "--verbose" ];
      modifyConfig = old: lib.recursiveUpdate old {
        target.${defaultRustTargetInfo.name} = {
          rustflags = (old.target.${defaultRustTargetInfo.name}.rustflags or []) ++ [
            "-C" "link-arg=-lc"
            "-C" "link-arg=-L${libcDir}/lib"

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
  };

  # canAutomate = true;
}
