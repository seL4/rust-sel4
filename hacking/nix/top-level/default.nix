self: with self;

let

  aggregate = name: drvs: pkgs.build.writeText "aggregate-${name}" (toString (lib.flatten drvs));

in {

  generatedSources = {
    inherit (pkgs.build.this.generatedCargoManifests) update check;
  };

  worldsForEverythingInstances = [
    pkgs.host.x86_64.none.this.worlds.default
    pkgs.host.aarch64.none.this.worlds.default
    pkgs.host.aarch64.none.this.worlds.qemu-arm-virt.sel4cp
  ];

  sel4testInstances = (map (x: x.this.sel4test) [
    pkgs.host.aarch64.linux
    pkgs.host.aarch32.linux
    pkgs.host.riscv64.noneWithLibc
    pkgs.host.riscv32.noneWithLibc
    pkgs.host.x86_64.linux
    pkgs.host.ia32.linux
  ]);

  prerequisites = aggregate "prerequisites" [
    pkgs.host.riscv64.noneWithLibc.gccMultiStdenvGeneric
    pkgs.build.this.qemuForSeL4
    pkgs.build.this.cargoManifestGenrationUtils.rustfmtWithTOMLSupport
    pkgs.build.this.capdl-tool
    pkgs.build.this.vendoredTopLevelLockfile.vendoredSourcesDirectory

    # unecessary
    generatedSources.check
  ];

  incremental = aggregate "incremental" [
    (lib.forEach worldsForEverythingInstances (world:
      map (instance: instance.links) world.instances.all
    ))

    sel4testInstances

    pkgs.host.aarch32.none.this.worlds.default.seL4
    pkgs.host.riscv64.none.this.worlds.default.seL4
    pkgs.host.riscv32.none.this.worlds.default.seL4
    pkgs.host.ia32.none.this.worlds.default.seL4

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

  slowTests = map (x: x.automate) sel4testInstances;

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
  } // lib.listToAttrs
    (lib.forEach
      [ "aarch64" "riscv64" "x86_64" ]
      (arch: lib.nameValuePair arch (pkgs.host.${arch}.none.this.worlds)))
  );

  example = worlds.default.instances.examples.root-task.example-root-task.simulate;

  example-rpi4-b-4gb = worlds.aarch64.bcm2711.instances.examples.root-task.example-root-task.bootCopied;

}
