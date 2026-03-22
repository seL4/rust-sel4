#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ topLevel
}:

let
  inherit (topLevel) lib pkgs;
  inherit (pkgs) build;
  inherit (build) writers linkFarm this;
  inherit (this) crateUtils;

  targetRootDir = toString ../../../target;

  toolchainPath = ../../../rust-toolchain.toml;

  rustupTargets = (lib.importTOML toolchainPath).toolchain.targets;

  isRustupTarget = targetName: lib.elem targetName rustupTargets;

  targetsPath = ../../../support/targets;

  customTargets =
    let
      parseTargetName = fname:
        let
          m = builtins.match "(.*)\\.json" fname;
        in
          if m == null then null else lib.elemAt m 0
      ;
      targetNames = lib.filter (x: x != null) (map parseTargetName (lib.attrNames(builtins.readDir targetsPath)));
    in
      targetNames
    ;

  hasSegment = seg: targetName: lib.elem seg (lib.splitString "-" targetName);

  firstSegment = targetName: lib.head (lib.splitString "-" targetName);

  getCCExePath = stdenv:
    let
      inherit (stdenv) cc;
    in
      "${cc}/bin/${cc.targetPrefix}gcc";

  getNewlibDir = stdenv: "${stdenv.cc.libc}/${stdenv.hostPlatform.config}";

  mkIncludeArg = d: "-I${d}/include";

  builtinMuslTargets = [
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
    "armv7-unknown-linux-musleabi"
    "riscv64gc-unknown-linux-musl"
    "riscv32gc-unknown-linux-musl"
  ];

  builtinBareMetalTargets = [
    "x86_64-unknown-none"
    "aarch64-unknown-none"
    "armv7a-none-eabi"
    "riscv64imac-unknown-none-elf"
    "riscv64gc-unknown-none-elf"
    "riscv32imac-unknown-none-elf"
    "riscv32imafc-unknown-none-elf"
  ];

  allTargets = builtinMuslTargets ++ builtinBareMetalTargets ++ customTargets;

  hasMusl = hasSegment "musl";

  hasSeL4 = targetName: hasSegment "sel4" targetName || hasSegment "microkit" targetName;

  getPkgsForTarget = target: {
    "x86_64" = pkgs.host.x86_64.none;
    "aarch64" = pkgs.host.aarch64.none;
    "armv7" = pkgs.host.aarch32.none;
    "armv7a" = pkgs.host.aarch32.none;
    "riscv64gc" = pkgs.host.riscv64.gc.none;
    "riscv64imac" = pkgs.host.riscv64.imac.none;
    "riscv32gc" = pkgs.host.riscv32.gc.none;
    "riscv32imac" = pkgs.host.riscv32.imac.none;
    "riscv32imafc" = pkgs.host.riscv32.imafc.none;
  }.${firstSegment target};

  ccConfigForTarget = target:
    let
      hostPkgs = getPkgsForTarget target;
      inherit (hostPkgs) stdenv;
      inherit (hostPkgs.this) muslForSeL4 dummyLibunwind;
    in
      crateUtils.clobber ([
        {
          env = {
            "CC_${target}" = getCCExePath stdenv;
          };
        }
        (
          if hasSeL4 target && hasMusl target
          then {
            env = {
              "CFLAGS_${target}" = "-nostdlib ${mkIncludeArg muslForSeL4}";
              "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg muslForSeL4;
            };
            target.${target} = {
              rustflags = [
                "-L${muslForSeL4}/lib"
                "-L${dummyLibunwind}/lib"
              ];
            };
          }
          else {
            env = {
              "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg (getNewlibDir stdenv);
            };
          }
        )
      ])
    ;

  ccConfigCommon = {
    env = {
      HOST_CC = getCCExePath build.stdenv;
      LIBCLANG_PATH = build.this.libclangPath;
    };
  };

  cc = writers.writeTOML "cc.toml" (crateUtils.clobber ([
    ccConfigCommon
  ] ++ map ccConfigForTarget allTargets));

  byTarget = lib.genAttrs allTargets (target:
    writers.writeTOML "${target}.toml" (crateUtils.clobber ([
      {
        build = {
          target = target;
        };
      }
      (lib.optionalAttrs (!(isRustupTarget target)) {
        unstable = {
          build-std = [ "compiler_builtins" "core" "alloc" ] ++ lib.optional (hasMusl target) "std";
          build-std-features = [ "compiler-builtins-mem" ];
        };
      })
      ccConfigCommon
      (ccConfigForTarget target)
    ]))
  );

  byTargetLinks = linkFarm "by-target" (lib.flip lib.mapAttrs' byTarget (k: v: lib.nameValuePair ("${k}.toml") v));

  worlds = lib.mapAttrs
    (_: attrs: attrs.none.this.worlds or attrs.default.none.this.worlds /* HACK */)
    pkgs.host
  ;

  configForWorld = attrPath: world:
    {
      build.target-dir = "${targetRootDir}/by-world/${lib.concatStringsSep "." attrPath}";
      env = world.seL4RustEnvVars;
    };

  byWorldList = lib.mapAttrsToListRecursiveCond
    (_: attrs: !(attrs.__isWorld or false))
    (attrPath: world: {
      path = attrPath;
      config = configForWorld attrPath world;
    })
    worlds
  ;

  byWorldLinks = linkFarm "by-world"
    (lib.listToAttrs
      (map
        ({ path, config }:
          let
            name = "${lib.concatStringsSep "." path}.toml";
          in
            lib.nameValuePair
              name
              (writers.writeTOML name config))
        byWorldList));

  links = linkFarm "generated-config" {
    "cc.toml" = cc;
    "by-target" = byTargetLinks;
    "by-world" = byWorldLinks;
  };

in {

  inherit links cc;

  utils = {
    inherit
      firstSegment
      hasSegment
      customTargets
      builtinBareMetalTargets
    ;
  };
}

# -nostdlib
