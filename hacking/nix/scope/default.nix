#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPlatform, hostPlatform, targetPlatform
, pkgsBuildBuild
, callPackage
, runCommand, linkFarm
, makeWrapper
, overrideCC, libcCross
, fetchurl
, qemu
}:

let
  superCallPackage = callPackage;
in

self:

let
  callPackage = self.callPackage;
in

let
  # HACK: unify across cross pkgsets
  callBuildBuildPackage = self.otherSplices.selfBuildBuild.callPackage;
in

let
  fenixRev = "9af557bccdfa8fb6a425661c33dbae46afef0afa";
  fenixSource = fetchTarball "https://github.com/nix-community/fenix/archive/${fenixRev}.tar.gz";
  fenix = import fenixSource {};
in

superCallPackage ../rust-utils {} self //

(with self; {

  sources = callPackage ./sources.nix {};

  seL4Arch = with hostPlatform;
    if isAarch64 then "aarch64" else
    if isAarch32 then "aarch32" else
    if isRiscV64 then "riscv64" else
    if isRiscV32 then "riscv32" else
    if isx86_64 then "x86_64" else
    if isx86_32 then "ia32" else
    throw "unkown platform";

  ### rust

  inherit fenix;

  topLevelRustToolchainFile = ../../../rust-toolchain.toml;

  defaultRustToolchain = fenix.fromToolchainFile {
    file = topLevelRustToolchainFile;
    sha256 = "sha256-GJR7CjFPMh450uP/EUzXeng15vusV3ktq7Cioop945U=";
  };

  upstreamDefaultRustEnvironment = elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
    rustToolchain = defaultRustToolchain;
  } // {
    channel = (builtins.fromTOML (builtins.readFile topLevelRustToolchainFile)).toolchain.channel;
    compilerRTSource = mkCompilerRTSource {
      version = "19.1-2024-07-30";
      hash = "sha256-fV51iDAbkRmWJj0twTmQKdZdLueMAKSZR6bBtgVPCbk=";
    };
    mkCustomTargetPath = customTargetTripleTripleName:
      let
        fname = "${customTargetTripleTripleName}.json";
      in
        linkFarm "targets" [
          { name = fname; path = sources.srcRoot + "/support/targets/${fname}"; }
        ];
  });

  ferroceneDefaultRustEnvironment = ferrocene.rustEnvironment;

  defaultRustEnvironment = upstreamDefaultRustEnvironment;
  # defaultRustEnvironment = ferroceneDefaultRustEnvironment;

  mkDefaultElaborateRustEnvironmentArgs = { rustToolchain }: rec {
    inherit rustToolchain;

    chooseLinker = { targetTriple, platform, cc }:
      if platform.config == buildPlatform.config
      then null
      else (
        if platform.isNone
        then "${rustToolchain}/lib/rustlib/${buildPlatform.config}/bin/rust-lld"
        else "${cc.targetPrefix}cc"
      );

    vendoredSuperLockfile = vendoredTopLevelLockfile;
  };

  topLevelLockfile = sources.srcRoot + "/Cargo.lock";

  vendoredTopLevelLockfile = vendorLockfile { lockfile = topLevelLockfile; };

  rustTargetArchName = {
    aarch64 = "aarch64";
    aarch32 = "armv7a";
    riscv64 = "riscv64${hostPlatform.this.rustTargetRiscVArch}";
    riscv32 = "riscv32${hostPlatform.this.rustTargetRiscVArch}";
    x86_64 = "x86_64";
    ia32 = "i686";
  }."${seL4Arch}";

  mkSeL4CustomRustTargetTripleName = { microkit ? false, resettable ? false, minimal ? false }:
    lib.concatStrings [
      rustTargetArchName
      "-sel4"
      (lib.optionalString microkit "-microkit")
      (lib.optionalString resettable "-resettable")
      (lib.optionalString minimal "-minimal")
    ];

  mkSeL4RustTargetTriple = args: mkCustomRustTargetTriple (mkSeL4CustomRustTargetTripleName args);

  bareMetalBuiltinRustTargetTriple = {
    aarch64 = "aarch64-unknown-none";
    aarch32 = "armv7a-none-eabi"; # armv7a-none-eabihf?
    riscv64 = "riscv64${hostPlatform.this.rustTargetRiscVArch}-unknown-none-elf";
    riscv32 = "riscv32${hostPlatform.this.rustTargetRiscVArch}-unknown-none-elf";
    x86_64 = "x86_64-unknown-none";
    ia32 = "i686-unknown-linux-gnu"; # HACK
  }."${seL4Arch}";

  bareMetalRustTargetTriple = mkBuiltinRustTargetTriple bareMetalBuiltinRustTargetTriple;

  defaultRustTargetTriple =
    if hostPlatform.isNone
    then mkSeL4RustTargetTriple {}
    else mkBuiltinRustTargetTriple hostPlatform.config;

  mkMkCustomTargetPathForEnvironment = { rustEnvironment }:
    let
      tool = buildCratesInLayers rec {
        inherit rustEnvironment;
        rootCrate = crates.sel4-generate-target-specs;
        lastLayerModifications = crateUtils.elaborateModifications {
          # HACK
          modifyDerivation = drv: drv.overrideAttrs (self: super: {
            nativeBuildInputs = (super.nativeBuildInputs or []) ++ [ makeWrapper ];
            postBuild = ''
              wrapProgram $out/bin/${rootCrate.name} \
                --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ rustEnvironment.rustToolchain ]}
            '';
          });
        };
      };

      dir = runCommand "target" {
        nativeBuildInputs = [ tool ];
      } ''
        mkdir $out
        sel4-generate-target-specs write --target-dir $out --all
      '';
    in
      customTargetTripleTripleName:
        let
          fname = "${customTargetTripleTripleName}.json";
        in
          linkFarm "targets" [
            { name = fname; path = builtins.toFile fname (builtins.readFile "${dir}/${fname}"); }
          ];

  inherit (callPackage ./crates.nix {})
    crates overridesForMkCrate globalPatchSection publicCrates publicCratesTxt;

  distribution = callPackage ./distribution.nix {};

  publicCratesCargoLock = pruneLockfile {
    vendoredSuperLockfile = vendoredTopLevelLockfile;
    rootCrates = lib.attrValues publicCrates;
    extraManifest = {
      patch = globalPatchSection;
    };
  };

  # HACK: reduce closure size, llvm now depends on targetPackage
  libclangPath = "${lib.getLib pkgsBuildBuild.llvmPackages.libclang}/lib";

  ### upstream tools

  capdl-tool = callBuildBuildPackage ./capdl-tool {};

  kani = callBuildBuildPackage ./kani {};

  verus = callBuildBuildPackage ./verus {};

  dafny = callBuildBuildPackage ./dafny {};

  ferrocene = callBuildBuildPackage ./ferrocene {};

  ### local tools

  mkTool = rootCrate: buildCratesInLayers {
    inherit rootCrate;
  };

  sel4-backtrace-embedded-debug-cli = mkTool crates.sel4-backtrace-embedded-debug-info-cli;
  sel4-backtrace-cli = mkTool crates.sel4-backtrace-cli;
  sel4-capdl-initializer-add-spec = mkTool crates.sel4-capdl-initializer-add-spec;
  sel4-simple-task-runtime-config-cli = mkTool crates.sel4-simple-task-runtime-config-cli;
  sel4-kernel-loader-add-payload = mkTool crates.sel4-kernel-loader-add-payload;
  sel4-reset-cli = mkTool crates.sel4-reset-cli;

  prepareResettable = callPackage ./prepare-resettable.nix {};
  embedDebugInfo = callPackage ./embed-debug-info.nix {};

  shellForMakefile = callPackage ./shell-for-makefile.nix {};
  shellForHacking = callPackage ./shell-for-hacking.nix {};

  ### unit tests

  someUnitTests = buildCratesInLayers {
    name = "some-unit-tests";
    test = true;
    rootCrates = with crates; [
      sel4-bitfield-ops
      sel4-kernel-loader-embed-page-tables
      sel4-backtrace-types
    ];
    features = [
      "sel4-backtrace-types/full"
    ];
  };

  ### kernel

  mkSeL4 = callPackage ./sel4 {};

  mkMicrokit = callPackage ./microkit {};

  cmakeConfigHelpers = callPackage ./cmake-config-helpers.nix {};

  ### worlds

  overrideWorldScope = self: super: {};

  mkWorldFrom = newScope: unelaboratedWorldConfig: (lib.makeScope newScope (callPackage ./world {} unelaboratedWorldConfig)).overrideScope overrideWorldScope;

  mkWorld = mkWorldFrom newScope;

  worlds = (callPackage ./worlds.nix {})."${seL4Arch}";

  platUtils = callPackage ./plat-utils {};

  ### sel4test

  mkSeL4Test = callPackage ./sel4test {};

  # TODO name more configurations
  sel4test = makeOverridable' mkSeL4Test {
    # mcs = true;
  };

  ### helpers

  # Like to `lib.callPackageWith`, except without `lib.makeOverridable`.
  callWith = autoArgs: fn: args:
    let
      f = if lib.isFunction fn then fn else import fn;
      auto = builtins.intersectAttrs (lib.functionArgs f) autoArgs;
    in f (auto // args);

  # Like `lib.makeOverridable`, except it adds an orthogonal dimension of overrideablility
  # accessible at `.override'`.
  makeOverridable' = f: origArgs:
    let
      overrideWith = newArgs: origArgs // (if lib.isFunction newArgs then newArgs origArgs else newArgs);
    in f origArgs // {
      override' = newArgs: makeOverridable' f (overrideWith newArgs);
    };

  ### QEMU

  opensbi = callPackage ./opensbi.nix {};

  qemuForSeL4 = callPackage ./qemu {};
  qemuForSeL4Xilinx = callPackage ./qemu/xilinx.nix {};

})
