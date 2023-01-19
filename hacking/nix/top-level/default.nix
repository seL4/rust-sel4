self: with self; {

  worldsForEverythingInstances = [
    pkgs.host.aarch64.none.this.worlds.default
    pkgs.host.x86_64.none.this.worlds.default
  ];

  everything = lib.flatten [
    (lib.forEach worldsForEverythingInstances (world:
      map (instance: instance.links) world.instances.supported
    ))
    pkgs.host.riscv64.none.this.worlds.default.kernel
  ];

  example = pkgs.host.aarch64.none.this.worlds.default.instances.examples.full-runtime.simulate;

  worlds.default = pkgs.host.aarch64.none.this.worlds.default;

  runAutomatedTests = mkRunAutomatedTests
    (lib.flatten
      (lib.forEach worldsForEverythingInstances (world:
        lib.forEach (lib.filter (instance: instance.canAutomate) world.instances.supported) (instance: {
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

}
