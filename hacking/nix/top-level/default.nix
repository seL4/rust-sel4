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

  example = pkgs.host.aarch64.none.this.worlds.default.instances.examples.full-runtime.run;

  worlds.default = pkgs.host.aarch64.none.this.worlds.default;

}
