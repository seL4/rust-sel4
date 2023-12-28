#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPlatform, hostPlatform, targetPlatform
, pkgsBuildBuild
, callPackage
, linkFarm
, overrideCC, libcCross
, treeHelpers
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
  fenixRev = "260b00254fc152885283b0d2aec78547a1f77efd";
  fenixSource = fetchTarball "https://github.com/nix-community/fenix/archive/${fenixRev}.tar.gz";
  fenix = import fenixSource {};

  rustToolchainParams = {
    channel = "nightly";
    date = "2023-08-02";
    sha256 = "sha256-LMK50izxGqjJVPhVG+C2VosDHvDYNXhaOFSb8fwGf/Y=";
  };

  mkRustToolchain = target: fenix.targets.${target}.toolchainOf rustToolchainParams;

  # TODO
  # rustToolchain = fenix.combine ([
  #   (mkRustToolchain hostPlatform.config).completeToolchain
  # ] ++ lib.optionals (hostPlatform.config != targetPlatform.config && !targetPlatform.isNone) [
  #   (mkRustToolchain targetPlatform.config).rust-std
  # ]);

  rustToolchain = (mkRustToolchain buildPlatform.config).completeToolchain;

in

superCallPackage ../rust-utils {} self //

(with self; {

  sources = callPackage ./sources.nix {};

  seL4Arch =
    lib.head
      (map
        (x: lib.elemAt x 1)
        (lib.filter
          (x: lib.elemAt x 0)
          (with hostPlatform; [
            [ isAarch64 "aarch64" ]
            [ isAarch32 "aarch32" ]
            [ isRiscV64 "riscv64" ]
            [ isRiscV32 "riscv32" ]
            [ isx86_64 "x86_64" ]
            [ isx86_32 "ia32" ]
          ])));

  ### rust

  defaultRustToolchain = rustToolchain;

  rustTargetArchName = {
    aarch64 = "aarch64";
    aarch32 = "armv7a";
    riscv64 = "riscv64imac";
    riscv32 = "riscv32imac";
    x86_64 = "x86_64";
    ia32 = "i686";
  }."${seL4Arch}";

  defaultRustTargetInfo =
    if !hostPlatform.isNone
    then mkBuiltinRustTargetInfo hostPlatform.config
    else seL4RustTargetInfoWithConfig {};

  seL4RustTargetInfoWithConfig = { microkit ? false, minimal ? false }: mkCustomRustTargetInfo "${rustTargetArchName}-sel4${lib.optionalString microkit "-microkit"}${lib.optionalString minimal "-minimal"}";

  bareMetalRustTargetInfo = mkBuiltinRustTargetInfo {
    aarch64 = "aarch64-unknown-none";
    aarch32 = "armv7a-none-eabi"; # armv7a-none-eabihf?
    riscv64 = "riscv64imac-unknown-none-elf"; # gc?
    riscv32 = "riscv32imac-unknown-none-elf"; # gc?
    x86_64 = "x86_64-unknown-none";
    ia32 = "i686-unknown-linux-gnu"; # HACK
  }."${seL4Arch}";

  mkBuiltinRustTargetInfo = name: {
    inherit name;
    path = null;
  };

  mkCustomRustTargetInfo = name: {
    inherit name;
    path =
      let
        fname = "${name}.json";
      in
        linkFarm "targets" [
          { name = fname; path = sources.srcRoot + "/support/targets/${fname}"; }
        ];
  };

  chooseLinkerForRustTarget = { rustToolchain, rustTargetName, platform }:
    if platform.isNone
    then "${rustToolchain}/lib/rustlib/${buildPlatform.config}/bin/rust-lld"
    else null;

  inherit (callPackage ./crates.nix {}) crates publicCrates publicCratesTxt;

  distribution = callPackage ./distribution.nix {};

  buildCrateInLayersHere = buildCrateInLayers {
    # TODO pass vendored lockfile instead
    superLockfile = topLevelLockfile;
  };

  topLevelLockfile = sources.srcRoot + "/Cargo.lock";

  vendoredTopLevelLockfile = vendorLockfile { lockfile = topLevelLockfile; };

  publicCratesCargoLock = pruneLockfile {
    superLockfile = topLevelLockfile;
    superLockfileVendoringConfig = vendoredTopLevelLockfile.configFragment;
    rootCrates = lib.attrValues publicCrates;
  };

  # HACK: reduce closure size, llvm now depends on targetPackage
  libclangPath = "${lib.getLib pkgsBuildBuild.llvmPackages.libclang}/lib";

  ### upstream tools

  capdl-tool = callBuildBuildPackage ./capdl-tool {};

  ### local tools

  mkTool = rootCrate: buildCrateInLayersHere {
    inherit rootCrate;
    release = false;
  };

  sel4-backtrace-embedded-debug-cli = mkTool crates.sel4-backtrace-embedded-debug-info-cli;
  sel4-backtrace-cli = mkTool crates.sel4-backtrace-cli;
  sel4-capdl-initializer-add-spec = mkTool crates.sel4-capdl-initializer-add-spec;
  sel4-simple-task-runtime-config-cli = mkTool crates.sel4-simple-task-runtime-config-cli;
  sel4-kernel-loader-add-payload = mkTool crates.sel4-kernel-loader-add-payload;

  embedDebugInfo = callPackage ./embed-debug-info.nix {};

  shellForMakefile = callPackage ./shell-for-makefile.nix {};
  shellForHacking = callPackage ./shell-for-hacking.nix {};

  ### kernel

  mkSeL4 = callPackage ./sel4 {};

  mkMicrokit = callPackage ./microkit {};

  cmakeConfigHelpers = callPackage ./cmake-config-helpers.nix {};

  ### worlds

  mkWorld = unelaboratedWorldConfig: lib.makeScope newScope (callPackage ./world {} unelaboratedWorldConfig);

  worlds = (callPackage ./worlds.nix {})."${seL4Arch}";

  platUtils = callPackage ./plat-utils {};

  ### sel4test

  mkSeL4Test = callPackage ./sel4test {};

  # TODO name more configurations
  sel4test = makeOverridable' mkSeL4Test {
    rust = hostPlatform.isAarch || hostPlatform.isRiscV || hostPlatform.isx86_64;
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

  qemuForSeL4 = (qemu.override {
    hostCpuTargets = [
      "arm-softmmu"
      "aarch64-softmmu"
      "riscv32-softmmu"
      "riscv64-softmmu"
      "i386-softmmu"
      "x86_64-softmmu"
    ];
    guestAgentSupport = false;
    numaSupport = false;
    seccompSupport = false;
    alsaSupport = false;
    pulseSupport = false;
    sdlSupport = false;
    jackSupport = false;
    gtkSupport = false;
    vncSupport = false;
    smartcardSupport = false;
    spiceSupport = false;
    ncursesSupport = false;
    usbredirSupport = false;
    libiscsiSupport = false;
    tpmSupport = false;
    uringSupport = false;
  }).overrideDerivation (attrs: {
    # patches from https://github.com/coliasgroup/qemu
    patches = attrs.patches ++ [
      # nspin/arm-virt-sp804
      (fetchurl {
        url = "https://github.com/coliasgroup/qemu/commit/79310d4cd22230a0dfca55697729670fe7e952fa.patch";
        sha256 = "sha256-6CMhLFo7B6tGrOfvIqfT+ZtJz7A7WBfHazeAYECDWbE=";
      })
      # nspin/opensbi-fw-payload-use-elf-entry-point
      (fetchurl {
        url = "https://github.com/coliasgroup/qemu/commit/4b0e8e5be4cdcdd9aeb387f949bbc8a1dbfe9299.patch";
        sha256 = "sha256-CoEnu5Ijy+khO7Jqq8NaKzJ1E4lLdaKDFF1ZC/I1C6k=";
      })
    ];
  });

})
