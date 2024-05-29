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

  defaultRustToolchain = fenix.fromToolchainFile {
    dir = ../../..;
    sha256 = "sha256-6lRcCTSUmWOh0GheLMTZkY7JC273pWLp2s98Bb2REJQ=";
  };

  rustTargetArchName = {
    aarch64 = "aarch64";
    aarch32 = "armv7a";
    riscv64 = "riscv64${hostPlatform.this.rustTargetRiscVArch}";
    riscv32 = "riscv32${hostPlatform.this.rustTargetRiscVArch}";
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
    riscv64 = "riscv64${hostPlatform.this.rustTargetRiscVArch}-unknown-none-elf";
    riscv32 = "riscv32${hostPlatform.this.rustTargetRiscVArch}-unknown-none-elf";
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

  inherit (callPackage ./crates.nix {}) crates globalPatchSection publicCrates publicCratesTxt;

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
    extraManifest = {
      patch = globalPatchSection;
    };
  };

  # HACK: reduce closure size, llvm now depends on targetPackage
  libclangPath = "${lib.getLib pkgsBuildBuild.llvmPackages.libclang}/lib";

  ### upstream tools

  capdl-tool = callBuildBuildPackage ./capdl-tool {};

  verus = callBuildBuildPackage ./verus {};

  dafny = callBuildBuildPackage ./dafny {};

  ### local tools

  mkTool = rootCrate: buildCrateInLayersHere {
    inherit rootCrate;
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
