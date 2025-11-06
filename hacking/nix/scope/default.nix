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
, overrideCC
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
  fenix =
    let
      rev = "320433651636186ea32b387cff05d6bbfa30cea7";
      source = builtins.fetchTarball {
        url = "https://github.com/nix-community/fenix/archive/${rev}.tar.gz";
        sha256 = "sha256:17c0075pcfvlhqn4lzjpacb3k44nfzm7w1qw4ivmmp6w1xmgzsqm";
      };
    in
      import source {};
in

let
  elaborateScopeConfig =
    { rustEnvironmentSelector ? {}
    , runClippyDefault ? false
    }:
    let
      elaborateRustEnvironmentSelector =
        { tracks ? "upstream"
        , upstream ? true
        , custom ? null
        }:
        {
          inherit
            tracks
            upstream
            custom
          ;
        };
    in {
      inherit
        runClippyDefault
      ;
      rustEnvironmentSelector = elaborateRustEnvironmentSelector rustEnvironmentSelector;
    };
in

superCallPackage ../rust-utils {} self //

(with self; {

  overridableScopeConfig = {};

  scopeConfig = elaborateScopeConfig overridableScopeConfig;

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

  topLevelRustToolchainFile = rec {
    path = ../../../rust-toolchain.toml;
    attrs = builtins.fromTOML (builtins.readFile path);
  };

  assembleRustToolchain = args:
    let
      toolchain = fenix.toolchainOf args;
      profile = topLevelRustToolchainFile.attrs.toolchain.profile or "default";
      explicitComponents = topLevelRustToolchainFile.attrs.toolchain.components;
      allComponents = toolchain.manifest.profiles.${profile} ++ explicitComponents;
      filteredComponents = lib.filter (component: toolchain ? ${component}) allComponents;
    in
      toolchain.withComponents filteredComponents;

  parseStructuredChannel = unstructuredChannel:
    let
      parts = builtins.match ''(nightly)-(.*)'' unstructuredChannel;
    in
      if parts == null
      then { channel = unstructuredChannel; date = null; }
      else { channel = lib.elemAt parts 0; date = lib.elemAt parts 1; };

  defaultRustEnvironment =
    let
      inherit (scopeConfig.rustEnvironmentSelector) tracks upstream custom;
    in
      if custom != null
      then
        let
          inherit (custom) channel sha256;
        in
          lib.fix (self: elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
            rustToolchain = assembleRustToolchain
              (parseStructuredChannel channel // builtins.removeAttrs custom ["channel"]);
          } // {
            inherit channel;
            mkCustomTargetPath = mkMkCustomTargetPathForEnvironment {
              rustEnvironment = self;
            };
          }))
      else {
        upstream = assert upstream; defaultUpstreamRustEnvironment;
        ferrocene = if upstream then ferrocene.upstreamRustEnvironment else ferrocene.rustEnvironment;
        verus = assert upstream; verus.rustEnvironment;
      }.${tracks};

  defaultRustToolchain = defaultRustEnvironment.rustToolchain;

  defaultUpstreamRustEnvironment = elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
    rustToolchain = fenix.fromToolchainFile {
      file = topLevelRustToolchainFile.path;
      sha256 = "sha256-JG3819J5cu2G8O/ju++ba/Jb6+pjZQgV6hfiQbipvyI=";
    };
  } // {
    channel = topLevelRustToolchainFile.attrs.toolchain.channel;
    mkCustomTargetPath = customTargetTripleTripleName:
      let
        fname = "${customTargetTripleTripleName}.json";
      in
        linkFarm "targets" [
          { name = fname; path = sources.srcRoot + "/support/targets/${fname}"; }
        ];
  });

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

  mkSeL4CustomRustTargetTripleName =
    { microkit ? false
    , resettable ? false
    , minimal ? false
    , unwind ? false
    , musl ? false
    }:
    lib.concatStrings [
      rustTargetArchName
      "-sel4"
      (lib.optionalString microkit "-microkit")
      (lib.optionalString resettable "-resettable")
      (lib.optionalString minimal "-minimal")
      (lib.optionalString unwind "-unwind")
      (lib.optionalString musl "-musl")
    ];

  allCustomRustTargetTripleNames =
    lib.map
      mkSeL4CustomRustTargetTripleName
      (lib.cartesianProduct
        (lib.mapAttrs
          (_: _: [ true false ])
          (lib.functionArgs mkSeL4CustomRustTargetTripleName)));

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
      tool = mkGenerateTargetSpecs {
        inherit rustEnvironment;
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

  mkGenerateTargetSpecs = callBuildBuildPackage ./generate-target-specs.nix {};

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

  cmakeConfigHelpers = callPackage ./cmake-config-helpers.nix {};

  mkMicrokit = callPackage ./microkit {};

  sdfgen = callPackage ./microkit/sdfgen.nix {};

  ### worlds

  overrideWorldScope = self: super: {};

  mkWorldFrom = newScope: unelaboratedWorldConfig: (lib.makeScope newScope (callPackage ./world {} unelaboratedWorldConfig)).overrideScope overrideWorldScope;

  mkWorld = mkWorldFrom newScope;

  worlds = (callPackage ./worlds.nix {})."${seL4Arch}";

  platUtils = callPackage ./plat-utils {};

  ### musl

  muslForSeL4 = callPackage ./musl {};

  muslForSeL4Raw = callPackage ./musl/raw.nix {};

  dummyLibunwind = callPackage ./musl/dummy-libunwind.nix {};

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
