#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

self: with self;

let

  aggregate = name: drvs: pkgs.build.writeText "aggregate-${name}" (toString (lib.flatten drvs));

in {

  inherit (pkgs.build.this) shellForMakefile shellForHacking;

  worldsForEverythingInstances = [
    pkgs.host.aarch64.none.this.worlds.default
    pkgs.host.aarch64.none.this.worlds.qemu-arm-virt.microkit
    pkgs.host.riscv64.default.none.this.worlds.default
    pkgs.host.riscv32.default.none.this.worlds.default
    pkgs.host.x86_64.none.this.worlds.default
  ];

  sel4testInstances = (lib.mapAttrs (k: v: v.this.sel4test.automate) {
    aarch64 = pkgs.host.aarch64.none;
    aarch32 = pkgs.host.aarch32.none;
    riscv64 = pkgs.host.riscv64.default.none;
    riscv32 = pkgs.host.riscv32.default.none;
    # TODO figure out why none doesn't build
    x86_64 = pkgs.host.x86_64.linux;
    ia32 = pkgs.host.ia32.linux; # no rust support yet
  });

  sel4testInstancesList = lib.attrValues sel4testInstances;

  prerequisites = aggregate "prerequisites" [
    pkgs.build.this.qemuForSeL4
    pkgs.build.this.capdl-tool
    pkgs.build.this.vendoredTopLevelLockfile.vendoredSourcesDirectory

    (lib.forEach (with pkgs.host; [ aarch64 aarch32 ]) (arch:
      arch.none.this.platUtils.rpi4.defaultBootLinks
    ))
  ];

  incremental = aggregate "incremental" [
    (lib.forEach worldsForEverythingInstances (world:
      map (instance: instance.links) world.instances.all
    ))

    sel4testInstancesList

    pkgs.host.aarch32.none.this.worlds.default.seL4
    pkgs.host.ia32.none.this.worlds.default.seL4

    pkgs.host.riscv64.imac.none.stdenv
    pkgs.host.riscv64.gc.none.stdenv
    pkgs.host.riscv32.imac.none.stdenv
    pkgs.host.riscv32.imafc.none.stdenv

    example
    example-rpi4-b-4gb
  ];

  nonIncremental = aggregate "non-incremental" [
    html
  ];

  excess = aggregate "excess" [
  ];

  everythingExceptNonIncremental = aggregate "everything-except-non-incremental" [
    prerequisites
    incremental
  ];

  everything = aggregate "everything" [
    everythingExceptNonIncremental
    nonIncremental
  ];

  everythingWithExcess = aggregate "everything-with-excess" [
    everything
    excess
  ];

  fastTests =
    lib.forEach worldsForEverythingInstances
      (world: world.instances.allAutomationScripts);

  slowTests = sel4testInstancesList;

  runFastTests = mkRunTests (lib.flatten [
    fastTests
  ]);

  witnessFastTests = mkWitnessTests (lib.flatten [
    fastTests
  ]);

  runTests = mkRunTests (lib.flatten [
    fastTests
    slowTests
  ]);

  witnessTests = mkWitnessTests (lib.flatten [
    fastTests
    slowTests
  ]);

  mkRunTests = scripts:
    with pkgs.build;
    writeScript "run-tests" ''
      #!${runtimeShell}
      set -eu

      ${lib.concatStrings (lib.forEach scripts (script: ''
        echo "<<< running test: ${script.testMeta.name or "unnamed"} >>>"
        ${script}
      ''))}

      echo
      echo '# All tests passed.'
      echo
    '';

  mkWitnessTest = script:
    pkgs.build.runCommand "witness-${script.testMeta.name or "unnamed"}" {} ''
      ${script}
      touch $out
    '';

  mkWitnessTests = scripts:
    pkgs.build.writeText "witness-tests" (lib.concatStrings (map mkWitnessTest scripts));

  docs = import ./docs {
    inherit lib pkgs;
  };

  inherit (docs) html;

  worlds = lib.fix (self: {
    default = self.aarch64.default;
  } // lib.mapAttrs (_: arch: arch.none.this.worlds) {
    inherit (pkgs.host) aarch64 aarch32 x86_64 i386;
    riscv64 = pkgs.host.riscv64.default;
    riscv32 = pkgs.host.riscv32.default;
  });

  example = worlds.default.instances.examples.root-task.example-root-task.simulate;

  example-rpi4-b-4gb = worlds.aarch64.bcm2711.instances.examples.root-task.example-root-task.bootCopied;

}
