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
  inherit (build) writers this;

  targetsPath = ../../../support/targets;

  seL4Targets =
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

  concatAttrs = lib.fold (x: y: x // y) {};
  clobberAttrs = lib.fold lib.recursiveUpdate {};

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

  getPkgsForTarget = target: {
    "x86_64" = pkgs.host.x86_64.none;
    "aarch64" = pkgs.host.aarch64.none;
    "armv7" = pkgs.host.aarch32.none;
    "armv7a" = pkgs.host.aarch32.none;
    "riscv64gc" = pkgs.host.riscv64.gc.none;
    "riscv64imac" = pkgs.host.riscv64.imac.none;
    "riscv32imac" = pkgs.host.riscv32.imac.none;
    "riscv32imafc" = pkgs.host.riscv32.imafc.none;
  }.${firstSegment target};

  configForTarget = isBuiltin: target:
    let
      hostPkgs = getPkgsForTarget target;
      inherit (hostPkgs) stdenv;
      inherit (hostPkgs.this) muslForSeL4 dummyLibunwind;
    in
      if !isBuiltin && hasSegment "musl" target
      then {
        env = {
          "CC_${target}" = getCCExePath stdenv;
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
          "CC_${target}" = getCCExePath stdenv;
          "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg (getNewlibDir stdenv);
        };
      }
    ;

in {
  cc = writers.writeTOML "config-cc.toml" (clobberAttrs ([
    {
      env = {
        HOST_CC = getCCExePath build.stdenv;
        LIBCLANG_PATH = build.this.libclangPath;
      };
    }
  ] ++ map (configForTarget true) builtinBareMetalTargets
    ++ map (configForTarget false) seL4Targets
  ));

  utils = {
    inherit
      firstSegment
      hasSegment
      clobberAttrs
      concatAttrs
      seL4Targets
      builtinBareMetalTargets
    ;
  };
}

# -nostdlib
