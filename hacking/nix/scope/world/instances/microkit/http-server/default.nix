{ lib, stdenv, hostPlatform, buildPackages
, linkFarm, symlinkJoin, writeText, writeScript, runCommand
, fetchgit
, parted, fatresize, dosfstools, kmod
, cmake, perl, python3, python3Packages
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

  diskImage = buildPackages.vmTools.runInLinuxVM (runCommand "disk-image" {
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
    cp -r --no-preserve=owner,mode ${content}/localhost x/
    find x/ -name '*\?' -delete
    cp -r x/* /mnt/

    umount /mnt

    min=$(fatresize --info $partition | sed -rn 's/Min size: (.*)$/\1/p')
    min_rounded_up=$(python3 -c "print(512 * ($min // 512 + 1))")
    total_disk_size=$(expr $min_rounded_up + 512)

    fatresize -v -s $min_rounded_up $partition

    real_img=real-disk.img
    touch $real_img
    truncate -s $total_disk_size $real_img
    parted -s $real_img mklabel msdos
    parted -s $real_img mkpart primary fat16 512B 100%
    dd if=$partition of=$real_img oflag=seek_bytes seek=512 count=$min_rounded_up

    losetup -d $dev

    mv $real_img $out/disk.img
  '');

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
      "-blockdev" "node-name=blkdev0,read-only=on,driver=file,filename=${diskImage}/disk.img"
    ];
  };
} // {
  inherit pds;
  inherit diskImage;
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
