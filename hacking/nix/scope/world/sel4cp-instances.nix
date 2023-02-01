{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, symlinkJoin, writeText, runCommand
, callPackage
, sel4cp
, mkTask
, srcRoot
, crates
, seL4ForUserspace
, crateUtils
, seL4RustTargetInfoWithConfig
}:

let
  mkPD = args: mkTask (args // rec {
    layers = [
      crateUtils.defaultIntermediateLayer
      {
        crates = [ "sel4" ];
        modifications = {
          modifyDerivation = drv: drv.overrideAttrs (self: super: {
            SEL4_PREFIX = seL4ForUserspace;
          });
        };
      }
    ];
    rustTargetInfo = seL4RustTargetInfoWithConfig { cp = true; minimal = true; };
  });

in {
  hello = rec {
    pds = {
      hello = mkPD rec {
        rootCrate = crates.sel4cp-hello;
        release = false;
      };
    };
    system = sel4cp.mkSystem {
      searchPath = "${pds.hello}/bin";
      systemXML = srcRoot + "/crates/examples/sel4cp/hello/hello.system";
    };
  };

  banscii = rec {
    pds = {
      pl011-driver = mkPD rec {
        rootCrate = crates.banscii-pl011-driver;
        release = false;
      };
      assistant = mkPD rec {
        rootCrate = crates.banscii-assistant;
        release = false;
      };
    };
    system = sel4cp.mkSystem {
      searchPath = symlinkJoin {
        name = "x";
        paths = [
          "${pds.assistant}/bin"
          "${pds.pl011-driver}/bin"
        ];
      };
      systemXML = srcRoot + "/crates/examples/sel4cp/banscii/banscii.system";
    };
  };
}
