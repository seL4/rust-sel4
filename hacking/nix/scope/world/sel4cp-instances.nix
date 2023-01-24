{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, writeText, runCommand
, callPackage
, sel4cp
, mkTask
, srcRoot
, crates
, seL4ForUserspace
, crateUtils
}:

let
  pds = {
    hello = mkTask rec {
      rootCrate = crates.sel4cp-hello;
      release = false;
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
      extraProfile = {
        panic = "abort";
        debug = 2;
      };
      lastLayerModifications = {
        # modifyConfig = old: lib.recursiveUpdate old {
        #   target.${rustTargetName}.rustflags = old.target.${rustTargetName}.rustflags or [] ++ [
        #     "-C" "link-args=--verbose"
        #   ];
        # };
      };
      rustTargetName = "aarch64-sel4cp";
      rustTargetPath =
        let
          fname = "${rustTargetName}.json";
        in
          linkFarm "targets" [
            { name = fname; path = srcRoot + "/support/targets/${fname}"; }
          ];
    };
  };

  system = sel4cp.mkSystem {
    searchPath = "${pds.hello}/bin";
    systemXML = srcRoot + "/crates/examples/sel4cp/hello/hello.system";
  };

in rec {
  inherit
    pds system
  ;
}
