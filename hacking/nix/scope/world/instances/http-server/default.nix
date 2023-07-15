{ lib, hostPlatform, buildPackages
, runCommand, writeScript
, fetchgit
, cpio

, crates
, crateUtils
, sources

, seL4RustEnvVars
, seL4Config
, worldConfig

, mkInstance
, mkCapDLRootTask
, mkTask
}:

let
  inherit (worldConfig) isCorePlatform;

  content = builtins.fetchGit {
    url = "https://github.com/seL4/website_pr_hosting";
    ref = "PR_280";
    rev = "0a579415c4837c96c4d4629e4b4d4691aaff07ca";
  };

  contentCPIO = runCommand "x.cpio" {
    nativeBuildInputs = [ cpio ];
  } ''
    cd ${content}/localhost \
      && find . -print -depth \
      | cpio -o -H newc > $out
  '';

  instance = mkInstance {
    rootTask = mkCapDLRootTask rec {
      # small = true;
      script = sources.srcRoot + "/crates/private/tests/capdl/http-server/cdl.py";
      config = {
        components = {
          example_component.image = passthru.test.elf;
          example_component.heap_size = 256 * 1048576; # 256MB
        };
      };
      passthru = {
        inherit contentCPIO;
        test = mkTask {
          rootCrate = crates.tests-capdl-http-server-components-test;
          layers = [
            crateUtils.defaultIntermediateLayer
            {
              crates = [
                "sel4-simple-task-runtime"
              ];
              modifications = {
                modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars);
              };
            }
          ];
        };
      };
    };
    extraPlatformArgs.extraQemuArgs = [
      "-device" "virtio-net-device,netdev=netdev0"
      "-netdev" "user,id=netdev0,hostfwd=tcp::8000-:80"

      "-device" "virtio-blk-device,drive=blkdev0"
      "-blockdev" "node-name=blkdev0,read-only=on,driver=file,filename=${contentCPIO}"
    ];
    isSupported = hostPlatform.isAarch64 && !isCorePlatform && seL4Config.PLAT == "qemu-arm-virt";
    canAutomate = false;
  };

  automate =
    let
      py = buildPackages.python3.withPackages (pkgs: [
        pkgs.pexpect
        pkgs.requests
      ]);
    in
      writeScript "automate" ''
        #!${buildPackages.runtimeShell}
        set -eu

        ${py}/bin/python3 ${./automate.py} ${instance.links}/simulate
      '';

  automateInIsolation = runCommand "test" {} ''
    ${automate}
    touch $out
  '';

in {
  inherit instance automate automateInIsolation;
}
