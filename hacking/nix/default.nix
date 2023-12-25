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
          riscv64 = {
            none = mkLeafWithGuard {
              config = "riscv64-none-elf";
            };
            linux = mkLeafWithGuard {
              config = "riscv64-unknown-linux-gnu";
            };
          };
          riscv32 = {
            none = mkLeafWithGuard {
              config = "riscv32-none-elf";
            };
            linux = mkLeafWithGuard {
              config = "riscv32-unknown-linux-gnu";
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

  this = makeOverridableWith lib.id mkThis baseArgs;

in
  this
