self: with self; {

  generatedSources = {
    inherit (pkgs.build.this.generatedCargoManifests) update check;
  };

  worldsForEverythingInstances = [
    pkgs.host.x86_64.none.this.worlds.default
    pkgs.host.aarch64.none.this.worlds.default
    pkgs.host.aarch64.none.this.worlds.qemu-arm-virt.sel4cp
  ];

  everythingList = lib.flatten [
    (lib.forEach worldsForEverythingInstances (world:
      map (instance: instance.links) world.instances.subsets.supported
    ))

    (map (x: x.this.sel4test) [
      pkgs.host.aarch64.linux
      pkgs.host.aarch32.linux
      pkgs.host.riscv64.noneWithLibc
      pkgs.host.riscv32.noneWithLibc
      pkgs.host.x86_64.linux
      pkgs.host.ia32.linux
    ])

    pkgs.host.aarch32.none.this.worlds.default.seL4
    pkgs.host.riscv64.none.this.worlds.default.seL4
    pkgs.host.riscv32.none.this.worlds.default.seL4
    pkgs.host.ia32.none.this.worlds.default.seL4

    pkgs.host.riscv64.noneWithLibc.gccMultiStdenvGeneric
  ];

  everythingWithExcessList = lib.flatten [
    everything
    html
  ];

  everything = pkgs.build.writeText "everything" (toString everythingList);
  everythingWithExcess = pkgs.build.writeText "everything" (toString everythingWithExcessList);

  runAutomatedTests = mkRunAutomatedTests
    (lib.flatten
      (lib.forEach worldsForEverythingInstances (world:
        lib.forEach world.instances.subsets.canAutomate (instance: {
          name = "unnamed"; # TODO
          script = instance.automate;
        })
      ))
    );

  mkRunAutomatedTests = tests:
    with pkgs.build;
    writeScript "run-all" ''
      #!${runtimeShell}
      set -e

      ${lib.concatStrings (lib.forEach tests ({ name, script }: ''
        echo "<<< running case: ${name} >>>"
        ${script}
      ''))}

      echo
      echo '# All tests passed.'
      echo
    '';

  docs = import ./docs.nix {
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

}
