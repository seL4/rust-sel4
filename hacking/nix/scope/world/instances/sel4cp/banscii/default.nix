{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, symlinkJoin, writeText, runCommand
, callPackage
, sel4cp
, mkTask
, sources
, crates
, crateUtils
, seL4RustTargetInfoWithConfig
, mkCorePlatformInstance
, worldConfig

, maybe
, canSimulate
, mkPD
}:

let
  pds = {
    pl011-driver = mkPD rec {
      rootCrate = crates.banscii-pl011-driver;
    };
    assistant = mkPD rec {
      rootCrate = crates.banscii-assistant;
      rustTargetInfo = seL4RustTargetInfoWithConfig { cp = true; minimal = false; };
    };
    artist = mkPD rec {
      rootCrate = crates.banscii-artist;
      extraProfile = {
        # For RSA key generation
        build-override = {
          opt-level = 2;
        };
      };
    };
  };

in
mkCorePlatformInstance {
  system = sel4cp.mkSystem {
    searchPath = symlinkJoin {
      name = "x";
      paths = [
        "${pds.pl011-driver}/bin"
        "${pds.assistant}/bin"
        "${pds.artist}/bin"
      ];
    };
    systemXML = sources.srcRoot + "/crates/examples/sel4cp/banscii/banscii.system";
  };
} // {
  inherit pds;
}
