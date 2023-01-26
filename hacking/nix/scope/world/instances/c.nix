{ lib, stdenv, hostPlatform, buildPackages
, writeScript, linkFarm
, overrideCC, libcCross
, which, strace
, llvmPackages

, crates
, mkTask
, defaultRustTargetName

, mk
}:

let
  stdenvWithLibc =
    let
      bintools = stdenv.cc.bintools.override {
        libc = libcCross;
        noLibc = false;
      };
    in
      stdenv.override {
        cc = stdenv.cc.override {
          libc = libcCross;
          noLibc = false;
          inherit bintools;
        };
      };

  stdenvWithLld = overrideCC stdenvWithLibc (stdenvWithLibc.cc.override {
    bintools = buildPackages.llvmPackages.bintools;
  });

  ccWrapper = writeScript "this-cc-wrapper" ''
    #!${buildPackages.runtimeShell}
    # env
    # which ${stdenvWithLld.cc.targetPrefix}cc
    # exit 1
    exec strace -f -e trace=file ${stdenvWithLld.cc.targetPrefix}cc $@
  '';

  instance = mk {
    rootTask = mkTask rec {
      rootCrate = crates.tests-root-task-c;
      release = false;
      stdenv = stdenvWithLld;
      lastLayerModifications = {
        modifyDerivation = drv: drv.overrideAttrs (self: super: {
          # NIX_DEBUG = 3;
          nativeBuildInputs = super.nativeBuildInputs ++ [
            which
            strace
          ];
        });
        modifyConfig = old: lib.recursiveUpdate old {
          target.${defaultRustTargetName} = {

            linker = "${stdenv.cc.targetPrefix}ld.lld";
            rustflags = (old.target.${defaultRustTargetName}.rustflags or []) ++ [
              "-C" "linker-flavor=ld"
              "-C" "link-arg=-lc"
            ];

            # NOTE
            # This should work, but it doesn't.
            # TODO
            # Investigate
            # linker = "${stdenv.cc.targetPrefix}cc";
            # # linker = ccWrapper;
            # rustflags = (old.target.${defaultRustTargetName}.rustflags or []) ++ [
            #   "-C" "linker-flavor=gcc"
            #   "-C" "link-arg=-nostartfiles"
            #   "-C" "default-linker-libraries=on"
            #   "-Z" "gcc-ld=lld"
            #   # "-C" "link-arg=-fuse-ld=lld"
            # ];
          };
        };
      };
    };
    isSupported = false;
  };

in {
  inherit instance;
}
