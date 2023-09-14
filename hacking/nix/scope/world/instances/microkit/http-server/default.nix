{ lib, stdenv, hostPlatform, buildPackages
, linkFarm, symlinkJoin, writeText, writeScript, runCommand
, fetchgit
, cpio
, cmake, perl, python3Packages
, microkit

, crates
, crateUtils
, sources

, seL4Modifications
, seL4Config
, worldConfig
, mkMicrokitInstance
, seL4RustTargetInfoWithConfig

, canSimulate
, mkPD
}:

let
  inherit (worldConfig) isMicrokit;

  rustTargetInfo = seL4RustTargetInfoWithConfig { microkit = true; };

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

  libcDir = "${stdenv.cc.libc}/${hostPlatform.config}";

  pds = {
    http-server = mkPD {
      rootCrate = crates.microkit-http-server-example-server;
      # layers = [
      #   crateUtils.defaultIntermediateLayer
      #   {
      #     crates = [
      #       "sel4-microkit"
      #     ];
      #     modifications = seL4Modifications;
      #   }
      # ];
      inherit rustTargetInfo;
      commonModifications = {
        modifyDerivation = drv: drv.overrideAttrs (self: super: {
          HOST_CC = "${buildPackages.stdenv.cc.targetPrefix}gcc";
          "BINDGEN_EXTRA_CLANG_ARGS_${rustTargetInfo.name}" = [ "-I${libcDir}/include" ];
          nativeBuildInputs = super.nativeBuildInputs ++ [
            cmake
            perl
            python3Packages.jsonschema
            python3Packages.jinja2
          ];
        });
      };
      lastLayerModifications = seL4Modifications;
    };
    sp804-driver = mkPD {
      rootCrate = crates.microkit-http-server-example-sp804-driver;
      inherit rustTargetInfo;
    };
    virtio-net-driver = mkPD {
      rootCrate = crates.microkit-http-server-example-virtio-net-driver;
      inherit rustTargetInfo;
    };
    virtio-blk-driver = mkPD {
      rootCrate = crates.microkit-http-server-example-virtio-blk-driver;
      inherit rustTargetInfo;
    };
  };

in
lib.fix (self: mkMicrokitInstance {
  system = microkit.mkSystem {
    searchPath = symlinkJoin {
      name = "x";
      paths = [
        "${pds.http-server}/bin"
        "${pds.sp804-driver}/bin"
        "${pds.virtio-net-driver}/bin"
        "${pds.virtio-blk-driver}/bin"
      ];
    };
    systemXML = sources.srcRoot + "/crates/examples/microkit/http-server/http-server.system";
  };
  extraPlatformArgs = lib.optionalAttrs canSimulate {
    extraQemuArgs = [
      "-device" "virtio-net-device,netdev=netdev0"
      "-netdev" "user,id=netdev0,hostfwd=tcp::8080-:80,hostfwd=tcp::8443-:443"

      "-device" "virtio-blk-device,drive=blkdev0"
      "-blockdev" "node-name=blkdev0,read-only=on,driver=file,filename=${contentCPIO}"
    ];
  };
} // {
  inherit pds;
  inherit contentCPIO;
} // lib.optionalAttrs canSimulate rec {
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
        ${py}/bin/python3 ${./automate.py} ${self.simulate}
      '';
})
