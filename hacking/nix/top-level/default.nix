#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

self: with self;

import ./overriding.nix self //
import ./aggregates.nix self //
{

  inherit (pkgs.build.this) shellForMakefile shellForHacking;

  docs = import ./docs {
    inherit lib pkgs;
  };

  inherit (docs) html;

  worlds = lib.fix (self: {
    default = self.aarch64.default;
    microkit = rec {
      default = aarch64;
      aarch64 = self.aarch64.microkitDefault;
      riscv64 = self.riscv64.microkitDefault;
    };
  } // lib.mapAttrs (_: arch: arch.none.this.worlds) {
    inherit (pkgs.host) aarch64 aarch32 x86_64 i386;
    riscv64 = pkgs.host.riscv64.default;
    riscv32 = pkgs.host.riscv32.default;
  });

  example = worlds.default.instances.examples.root-task.example-root-task.simulate;

  example-rpi4-b-4gb = worlds.aarch64.bcm2711.instances.examples.root-task.example-root-task.bootCopied;

  cargoConfig = import ./cargo-config.nix {
    topLevel = self;
  };

}
