#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, hostPlatform, buildPackages
, linkFarm, symlinkJoin, writeText, writeScript, runCommand
, fetchgit
, parted, fatresize, dosfstools, kmod
, cmake, perl, python3, python3Packages
, microkit

, crates
, globalPatchSection
, crateUtils
, sources

, seL4Modifications
, seL4Config
, worldConfig
, callPlatform
, mkSeL4RustTargetTriple

, canSimulate
, mkPD
}:

let
  inherit (worldConfig) isMicrokit;

  targetTriple = mkSeL4RustTargetTriple { microkit = true; };

  content = builtins.fetchGit {
    url = "https://github.com/seL4/website_pr_hosting";
    ref = "gh-pages";
    rev = "e320919be95968dc13e2122d97737b02a8a31997";
  };

  diskImage = mkDiskImage {};
  smallDiskImage = mkDiskImage { excludePatterns = [ "*.mp4" "*.pdf" ]; };

  vmTools = buildPackages.vmTools.override {
    # HACK
    requireKVM = false;
  };

  mkDiskImage =
    { maxIndividualFileSize ? null
    , excludePatterns ? null
    }:
    let
      contentSubdir = "PR_371";
    in
    vmTools.runInLinuxVM (runCommand "disk-image" {
      nativeBuildInputs = [ python3 kmod parted fatresize dosfstools ];
      preVM = ''
        mkdir scratch
        scratch=$(realpath scratch)
        QEMU_OPTS+=" -virtfs local,path=$scratch,security_model=none,mount_tag=scratch"
      '';
    } ''
      mkdir /tmp/scratch
      mount -t 9p scratch /tmp/scratch -o trans=virtio,version=9p2000.L,msize=131072
      cd scratch

      modprobe loop

      mkdir /mnt

      # HACK: fatresize segfaults when run on a filesystem that's not on a partition (?)

      img_size=500M
      img=disk.img
      touch $img
      truncate -s $img_size $img

      dev=$(losetup --find --show $img)

      parted -s $dev mklabel msdos
      parted -s $dev mkpart primary fat16 512B 100%

      partprobe $dev
      partition=''${dev}p1

      mkfs.vfat -F 16 $partition

      mount $partition /mnt

      # HACK:
      #  - some filesystem layer doesn't seem to like '?' in filename
      #  - rsync doesn't play nicely with some filesystem layer
      cp -r --no-preserve=owner,mode ${content}/${contentSubdir} x/
      find x/ -name '*\?' -delete
      ${lib.optionalString (excludePatterns != null) ''
        find x/ -type f \( ${
          lib.concatMapStringsSep " -o " (pat: "-name '${pat}'") excludePatterns
        } \) -delete
      ''}
      ${lib.optionalString (maxIndividualFileSize != null) ''
        find x/ -size +${maxIndividualFileSize} -delete
      ''}
      cp -r x/* /mnt/

      umount /mnt

      min=$(fatresize --info $partition | sed -rn 's/Min size: (.*)$/\1/p')
      min_rounded_up=$(python3 -c "print(512 * ($min // 512 + 1))")
      partition_size=$(python3 -c "print(max(64 << 20, $min_rounded_up))")
      total_disk_size=$(expr $partition_size + 512)

      # NOTE
      # Somehow $partition sometimes ends up smaller than $partition_size.
      # conv=notrunc works around this possibility.
      fatresize -vf -s $partition_size $partition

      real_img=real-disk.img
      touch $real_img
      truncate -s $total_disk_size $real_img
      parted -s $real_img mklabel msdos
      parted -s $real_img mkpart primary fat16 512B 100%

      dd if=$partition of=$real_img iflag=count_bytes oflag=seek_bytes seek=512 count=$partition_size conv=notrunc

      losetup -d $dev

      mv $real_img $out/disk.img
    '');

  libcDir = "${stdenv.cc.libc}/${hostPlatform.config}";

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
  inherit diskImage smallDiskImage;
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
