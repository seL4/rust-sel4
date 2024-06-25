#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

let

  defaultNixpkgsPath =
    let
      rev = "1811c4fec88995679397d6fa20f4f3395a0bebe5";
    in
      builtins.fetchTarball {
        url = "https://github.com/coliasgroup/nixpkgs/archive/refs/tags/keep/${builtins.substring 0 32 rev}.tar.gz";
        sha256 = "sha256:0ad2c7vlr9fidzjjg8szigfhmp1gvlf62ckd6cir8ymrxc93pby7";
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

  treeHelpers = import ./tree-helpers.nix { inherit lib; };

  makeOverridableWith = f: g: x: (g x) // {
    override = x': makeOverridableWith f g (f x' x);
  };

  crossSystems =
    with treeHelpers;
    {
      build = mkLeaf null;
      host =
        let
          # Avoid cache misses in cases where buildPlatform == hostPlatform
          guard = attrs:
            if attrs == builtins.intersectAttrs attrs this.pkgs.build.hostPlatform
            then null
            else attrs
          ;
          mkLeafWithGuard = attrs: mkLeaf (guard attrs);
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
                  gccParams = { arch = "rv64imafc_zicsr_zifencei"; abi = "ilp32f"; };
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

  baseArgs = selfThis: {
    nixpkgsArgsFor = crossSystem: {
      inherit crossSystem;
      overlays = [
        (self: super: {
          thisTopLevel = selfThis;
          inherit treeHelpers;
        })
        (import ./overlay)
      ];
    };
  };

  mkThis =
    with treeHelpers;
    args: lib.fix (self:
      let
        concreteArgs = args self;
        pkgs = untree (mapLeaves (crossSystem:
          nixpkgsFn (concreteArgs.nixpkgsArgsFor crossSystem)
        ) crossSystems);
      in {
        inherit lib pkgs;
      } // import ./top-level self);

  this = lib.fix (self: makeOverridableWith lib.id mkThis baseArgs // rec {
    overrideNixpkgsArgs = f: self.override (superArgs: selfBase:
      let
        concreteSuperArgs = superArgs selfBase;
      in
        concreteSuperArgs // {
          nixpkgsArgsFor = crossSystem: f (concreteSuperArgs.nixpkgsArgsFor crossSystem);
        }
    );
    withOverlays = overlays: self.overrideNixpkgsArgs (superNixpkgsArgs:
      superNixpkgsArgs // {
        overlays = superNixpkgsArgs.overlays ++ overlays;
      }
    );
  });

in
  this
