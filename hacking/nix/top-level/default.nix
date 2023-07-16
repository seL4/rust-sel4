self: with self; {

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

  everythingList = lib.flatten [
    (lib.forEach worldsForEverythingInstances (world:
      map (instance: instance.links) world.instances.all
    ))

    sel4testInstances

    pkgs.host.aarch32.none.this.worlds.default.seL4
    pkgs.host.riscv64.none.this.worlds.default.seL4
    pkgs.host.riscv32.none.this.worlds.default.seL4
    pkgs.host.ia32.none.this.worlds.default.seL4

    pkgs.host.riscv64.noneWithLibc.gccMultiStdenvGeneric

    example
    example-rpi4-b-4gb
  ];

  everythingWithExcessList = lib.flatten [
    everything
    html
  ];

  everythingElseForCIList = lib.flatten [
    html
    example
    example-rpi4-b-4gb
  ];

  everything = pkgs.build.writeText "everything" (toString everythingList);
  everythingWithExcess = pkgs.build.writeText "everything-with-excess" (toString everythingWithExcessList);
  everythingElseForCI = pkgs.build.writeText "everything-else-for-ci" (toString everythingElseForCIList);

  fastAutomatedTests =
    lib.concatLists
      (lib.forEach worldsForEverythingInstances
        (world: world.instances.allAutomationScripts));

  slowAutomatedTests = map (x: x.automate) sel4testInstances;

  runAutomatedTests = mkRunAutomatedTests (lib.concatLists [
    fastAutomatedTests
    slowAutomatedTests
  ]);

  witnessAutomatedTests = mkWitnessAutomatedTests (lib.concatLists [
    fastAutomatedTests
    slowAutomatedTests
  ]);

  mkRunAutomatedTests = scripts:
    with pkgs.build;
    writeScript "run-tests" ''
      #!${runtimeShell}
      set -eu

      ${lib.concatStrings (lib.forEach scripts (script: ''
        echo "<<< running case: ${script.testMeta.name or "unnamed"} >>>"
        ${script}
      ''))}

      echo
      echo '# All tests passed.'
      echo
    '';

  mkWitnessAutomatedTest = script:
    with pkgs.build;
    runCommand "witness-${script.testMeta.name or "unnamed"}" {} ''
      ${script}
      touch $out
    '';

  mkWitnessAutomatedTests = scripts:
    with pkgs.build;
    writeText "witness-tests" (lib.concatStrings (
      map mkWitnessAutomatedTest scripts
    ));

  docs = import ./docs {
    inherit lib pkgs;
  };

  inherit (docs) html;

  # convenience

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
