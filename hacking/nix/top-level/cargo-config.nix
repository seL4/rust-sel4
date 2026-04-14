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
  inherit (build) writers linkFarm writeShellApplication this llvm python312;
  inherit (this) sources crateUtils capdl-tool sdfgen;

  targetRootDir = toString ../../../target;

  toolchainPath = ../../../rust-toolchain.toml;

  rustupTargets = (lib.importTOML toolchainPath).toolchain.targets;

  isRustupTarget = targetName: lib.elem targetName rustupTargets;

  allTargets = builtinTargets ++ customTargets;

  builtinTargets = rustupTargets ++ [
    "riscv32gc-unknown-linux-musl"
  ];

  builtinMuslTargets = lib.filter hasMusl builtinTargets;

  builtinBareMetalTargets = lib.filter (target: !(hasMusl target)) builtinTargets;

  targetsPath = ../../../support/targets;

  customTargets =
    let
      parseTargetName = fname:
        let
          m = builtins.match "(.*)\\.json" fname;
        in
          if m == null then null else lib.elemAt m 0
      ;
      targetNames = lib.filter (x: x != null) (map parseTargetName (lib.attrNames (builtins.readDir targetsPath)));
    in
      targetNames
    ;

  firstSegment = targetName: lib.head (lib.splitString "-" targetName);

  hasSegment = seg: targetName: lib.elem seg (lib.splitString "-" targetName);

  hasMusl = hasSegment "musl";

  hasSeL4 = targetName: hasSegment "sel4" targetName || hasSegment "microkit" targetName;

  getPkgsForTarget = target: with pkgs.host; {
    "x86_64" = x86_64;
    "aarch64" = aarch64;
    "armv7" = aarch32;
    "armv7a" = aarch32;
    "riscv64gc" = riscv64.gc;
    "riscv64imac" = riscv64.imac;
    "riscv32gc" = riscv32.gc;
    "riscv32imac" = riscv32.imac;
    "riscv32imafc" = riscv32.imafc;
  }.${firstSegment target}.none;
  # TODO
  # ${if hasMusl target then "linuxMusl" else "none"}

  getCCExePath = stdenv:
    let
      inherit (stdenv) cc;
    in
      "${cc}/bin/${cc.targetPrefix}gcc";

  getNewlibDir = stdenv: "${stdenv.cc.libc}/${stdenv.hostPlatform.config}";

  mkIncludeArg = d: "-I${d}/include";

  ccConfigCommon = {
    env = {
      HOST_CC = getCCExePath build.stdenv;
      LIBCLANG_PATH = build.this.libclangPath;
    };
  };

  ccConfigForTarget = target:
    let
      hostPkgs = getPkgsForTarget target;
      inherit (hostPkgs) stdenv;
      inherit (hostPkgs.this) muslForSeL4 dummyLibunwind;
    in
      crateUtils.clobber ([
        (lib.optionalAttrs (!(hasMusl target) || hasSeL4 target) {
          env = {
            "CC_${target}" = getCCExePath stdenv;
          };
        })
        (
          if !(hasMusl target)
          then {
            env = {
              "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg (getNewlibDir stdenv); # TODO necessary?
            };
          }
          else if hasSeL4 target
          then {
            env = {
              "CFLAGS_${target}" = "-nostdlib ${mkIncludeArg muslForSeL4}"; # TODO necessary?
              "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg muslForSeL4; # TODO necessary?
            };
            target.${target} = {
              rustflags = [
                "-L${muslForSeL4}/lib"
                "-L${dummyLibunwind}/lib"
              ];
            };
          }
          # special case, not included in rustup
          else if target == "riscv32gc-unknown-linux-musl"
          then
            let
              thesePkgs = pkgs.host.riscv32.gc.linuxMusl;
            in {
              target.${target} = {
                rustflags = [
                  "-L${thesePkgs.stdenv.cc.cc}/lib/gcc/${thesePkgs.stdenv.hostPlatform.config}/${thesePkgs.stdenv.cc.version}"
                  "-L${thesePkgs.stdenv.cc.libc}/lib"
                  "-L${thesePkgs.libunwind}/lib"
                ];
              };
            }
          else {
          }
        )
      ])
    ;

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
    (lib.filterAttrs (n: _: n != "ia32") pkgs.host)
  ;

  configForWorld = attrPath: world:
    let
      targetDir = "${targetRootDir}/by-world/${lib.concatStringsSep "." attrPath}";
    in {
      build.target-dir = targetDir;
      env = world.seL4RustEnvVars;
    } // lib.optionalAttrs world.worldConfig.canSimulate {
      target."cfg(not(any()))".runner =
        let
          simulateScript =
            let script = world.simulateScript;
            in "${script}/bin/${script.name}";
          runScript = writeShellApplication {
            name = "run";
            runtimeInputs = [
              llvm
              capdl-tool
              (python312.withPackages (p: with p; [
                future six
                aenum sortedcontainers
                pyyaml pyelftools pyfdt
                sdfgen
              ]))
            ];
            text = ''
              PYTHONPATH="${toString ../../src/python}:${sources.pythonCapDLTool}:''${PYTHONPATH:-}" \
                cargo run -p sel4-test-runner -- \
                  --target-dir ${targetDir} \
                  --object-sizes ${world.objectSizes} \
                  --simulate-script ${simulateScript} \
                  ${if world.worldConfig.isMicrokit then ''
                    --microkit-sdk ${world.microkit.sdk} \
                    --microkit-board ${world.worldConfig.microkitConfig.board} \
                    --microkit-config ${world.worldConfig.microkitConfig.config} \
                  '' else ''
                    --kernel ${world.seL4} \
                  ''} \
                  "$@"
            '';
          };
        in
          "${runScript}/bin/${runScript.name}";
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
    "target" = byTargetLinks;
    "world" = byWorldLinks;
  };

in {
  inherit
    links
    cc
    byTarget
    byWorldList
  ;
}

# -nostdlib
