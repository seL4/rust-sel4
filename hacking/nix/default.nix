#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

let

  defaultNixpkgsPath =
    let
      rev = "0a44fc400a2ea73cee67c4effbae10b6bb254da8"; # based on a snapshot of nixos-25.05
    in
      builtins.fetchTarball {
        url = "https://github.com/coliasgroup/nixpkgs/archive/refs/tags/keep/${builtins.substring 0 32 rev}.tar.gz";
        sha256 = "sha256:126i4sxbrj462k6aig8df6792fs6y3nnqyc8pklzv3j103f8wl36";
      };

in

# let
#   defaultNixpkgsPath = ../../../x/nixpkgs;
# in

{ nixpkgsPath ? defaultNixpkgsPath
, nixpkgsFn ? import nixpkgsPath
, lib ? import (nixpkgsPath + "/lib")
}:

let

  treeHelpers = rec {
    mapLeafNodes = f: lib.mapAttrs (k: v:
      assert lib.isAttrs v;
      if v ? __leaf
      then f v
      else mapLeafNodes f v
    );

    mkLeaf = value: {
      __leaf = null;
      inherit value;
    };

    untree = mapLeafNodes (leafNode: leafNode.value);

    mapLeaves = f: mapLeafNodes (leafNode: mkLeaf (f leafNode.value));
  };

  makeOverridable' = f: arg: lib.fix (self: (f self arg) // {
    override = modifyArg: makeOverridable' f (modifyArg arg);
  });

in let

  isCrossSystemActuallyCross =
    let
      inherit (nixpkgsFn {}) hostPlatform;
    in
      crossSystem: crossSystem != builtins.intersectAttrs crossSystem hostPlatform;

  crossSystemTree =
    with treeHelpers;
    {
      build = mkLeaf null;
      host =
        let
          # Avoid cache misses in cases where buildPlatform == hostPlatform
          guard = crossSystem:
            if isCrossSystemActuallyCross crossSystem
            then crossSystem
            else null;
          mkLeafWithGuard = crossSystem: mkLeaf (guard crossSystem);
        in {
          aarch64 = {
            none = mkLeafWithGuard {
              config = "aarch64-none-elf";
            };
            linux = mkLeafWithGuard {
              config = "aarch64-unknown-linux-gnu";
            };
            linuxMusl = mkLeafWithGuard {
              config = "aarch64-unknown-linux-musl";
            };
          };
          aarch32 = {
            none = mkLeafWithGuard {
              config = "arm-none-eabi";
            };
            linux = mkLeafWithGuard {
              config = "armv7l-unknown-linux-gnueabihf";
            };
          };
          riscv64 = rec {
            default = imac;
            imac = {
              none = mkLeafWithGuard (rec {
                config = "riscv64-none-elf";
                gcc = this.gccParams;
                this = {
                  rustTargetRiscVArch = "imac";
                  gccParams = { arch = "rv64imac_zicsr_zifencei"; abi = "lp64"; };
                };
              });
            };
            # TODO
            # Currently incompatible with the "cc" crate. Must do something like
            # https://github.com/rust-lang/cc-rs/pull/460 (except for bare metal) or
            # https://github.com/rust-lang/cc-rs/pull/796.
            # TODO
            # Will require KernelRiscvExtD in sel4test.
            gc = {
              none = mkLeafWithGuard (rec {
                config = "riscv64-none-elf";
                gcc = {}; # equivalent to default, omitting means we can use cached binary
                this = {
                  rustTargetRiscVArch = "gc";
                  gccParams = { arch = "rv64gc"; abi = "lp64d"; };
                };
              });
              linux = mkLeafWithGuard {
                config = "riscv64-unknown-linux-gnu";
              };
            };
          };
          riscv32 = rec {
            default = imac;
            imac = {
              none = mkLeafWithGuard (rec {
                config = "riscv32-none-elf";
                gcc = this.gccParams;
                this = {
                  rustTargetRiscVArch = "imac";
                  gccParams = { arch = "rv32imac_zicsr_zifencei"; abi = "ilp32"; };
                };
              });
            };
            # TODO (see note for riscv64.gc)
            # TODO will require KernelRiscvExtF in sel4test
            imafc = {
              none = mkLeafWithGuard (rec {
                config = "riscv32-none-elf";
                gcc = {}; # equivalent to default, omitting means we can use cached binary
                this = {
                  rustTargetRiscVArch = "imafc";
                  gccParams = { arch = "rv32imafdc_zicsr_zifencei"; abi = "ilp32d"; };
                };
              });
              linux = mkLeafWithGuard {
                config = "riscv32-unknown-linux-gnu";
              };
            };
          };
          x86_64 = {
            none = mkLeafWithGuard {
              config = "x86_64-elf";
            };
            linux = mkLeafWithGuard {
              config = "x86_64-unknown-linux-gnu";
            };
          };
          ia32 = {
            none = mkLeafWithGuard {
              config = "i686-elf";
            };
            linux = mkLeafWithGuard {
              config = "i686-unknown-linux-gnu";
            };
          };
        };
    };

in let

  f = self: arg:
    let
      concreteArg = arg self;
      pkgs = treeHelpers.untree (treeHelpers.mapLeaves (crossSystem:
        nixpkgsFn (concreteArg.nixpkgsArgsForCrossSystem crossSystem)
      ) crossSystemTree);
    in {
      inherit lib pkgs;
    } // import ./top-level self // concreteArg.extraAttrs;

  baseArg = self: {
    nixpkgsArgsForCrossSystem = crossSystem: {
      inherit crossSystem;
      overlays = [
        (import ./overlay)
        (_: _: {
          selfTopLevel = self;
        })
      ];
      config = baseConfig;
    };
    extraAttrs = {};
  };

  baseConfig = {
    permittedInsecurePackages = [
      "dotnet-sdk-6.0.428"
      "dotnet-runtime-6.0.36"
    ];
  };

in
  makeOverridable' f baseArg
