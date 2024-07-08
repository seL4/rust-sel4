#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, buildPackages
, callPackage
, runCommand, writeScript
, fetchgit
, perl
, microkit

, crates
, globalPatchSection
, crateUtils
, sources

, seL4Modifications
, worldConfig
, callPlatform
, mkSeL4RustTargetTriple

, canSimulate
, mkPD
}:

let
  inherit (worldConfig) isMicrokit;

  mkDiskImage = callPackage ./mk-disk-image.nix {};

  filter = { src, maxIndividualFileSize ? null, excludePatterns ? null }:
    runCommand "filtered" {} ''
      cp -r --no-preserve=owner,mode ${src} $out
      find $out -name  -delete
      ${lib.optionalString (excludePatterns != null) ''
        find $out -type f \( ${
          lib.concatMapStringsSep " -o " (pat: "-name '${pat}'") excludePatterns
        } \) -delete
      ''}
      ${lib.optionalString (maxIndividualFileSize != null) ''
        find $out -size +${maxIndividualFileSize} -delete
      ''}
    '';

  rawContent = builtins.fetchGit {
    url = "https://github.com/seL4/website_pr_hosting";
    ref = "gh-pages";
    rev = "e320919be95968dc13e2122d97737b02a8a31997";
  };

  # HACK link structure requires website_pr_hosting/${contentSubdir}
  patchedContent =
    let
      contentSubdir = "PR_371";
    in
      runCommand "patched" {} ''
        cp -r --no-preserve=owner,mode ${rawContent}/${contentSubdir} $out
        mkdir -p $out/website_pr_hosting
        cp -r --no-preserve=owner,mode ${rawContent}/${contentSubdir} $out/website_pr_hosting/${contentSubdir}/
      '';

  mkThisDiskImage = filterArgs: mkDiskImage {
    src = filter ({
      src = patchedContent;
    } // filterArgs);
  };

  commonExcludePatterns = [ ''*\?'' ];

  largeDiskImage = mkThisDiskImage { excludePatterns = commonExcludePatterns; };
  smallDiskImage = mkThisDiskImage { excludePatterns = commonExcludePatterns ++ [ "*.mp4" "*.pdf" ]; };

  # diskImage = largeDiskImage;
  diskImage = smallDiskImage;

  targetTriple = mkSeL4RustTargetTriple { microkit = true; };

  pds = {
    http-server = mkPD {
      rootCrate = crates.microkit-http-server-example-server;
      release = true;
      # layers = [
      #   crateUtils.defaultIntermediateLayer
      #   {
      #     crates = [
      #       "sel4-microkit"
      #     ];
      #     modifications = seL4Modifications;
      #   }
      # ];
      inherit targetTriple;
      commonModifications = {
        modifyManifest = lib.flip lib.recursiveUpdate {
          patch.crates-io = {
            inherit (globalPatchSection.crates-io) ring;
          };
        };
        modifyDerivation = drv: drv.overrideAttrs (self: super: {
          HOST_CC = "${buildPackages.stdenv.cc.targetPrefix}gcc";
          nativeBuildInputs = (super.nativeBuildInputs or []) ++ [
            perl
          ];
        });
      };
      # lastLayerModifications = seL4Modifications;
    };
    pl031-driver = mkPD {
      rootCrate = crates.microkit-http-server-example-pl031-driver;
      release = true;
      inherit targetTriple;
    };
    sp804-driver = mkPD {
      rootCrate = crates.microkit-http-server-example-sp804-driver;
      release = true;
      inherit targetTriple;
    };
    virtio-net-driver = mkPD {
      rootCrate = crates.microkit-http-server-example-virtio-net-driver;
      release = true;
      inherit targetTriple;
    };
    virtio-blk-driver = mkPD {
      rootCrate = crates.microkit-http-server-example-virtio-blk-driver;
      release = true;
      inherit targetTriple;
    };
  };

in
lib.fix (self: callPlatform {
  system = microkit.mkSystem {
    searchPath = [
      "${pds.http-server}/bin"
      "${pds.pl031-driver}/bin"
      "${pds.sp804-driver}/bin"
      "${pds.virtio-net-driver}/bin"
      "${pds.virtio-blk-driver}/bin"
    ];
    systemXML = sources.srcRoot + "/crates/examples/microkit/http-server/http-server.system";
  };
  extraPlatformArgs = lib.optionalAttrs canSimulate {
    extraQEMUArgs = [
      "-device" "virtio-net-device,netdev=netdev0"
      "-netdev" "user,id=netdev0,hostfwd=tcp::8080-:80,hostfwd=tcp::8443-:443"

      "-device" "virtio-blk-device,drive=blkdev0"
      "-blockdev" "node-name=blkdev0,read-only=on,driver=file,filename=${diskImage}/disk.img"
    ];
  };
} // {
  inherit pds;
  inherit largeDiskImage smallDiskImage diskImage;
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
