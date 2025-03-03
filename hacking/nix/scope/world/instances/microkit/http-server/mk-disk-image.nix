#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, buildPackages
, runCommand
, vmTools
, kmod, parted, fatresize, dosfstools, python3
}:

let
  vmTools = buildPackages.vmTools.override {
    # HACK
    requireKVM = false;
  };

in

{ src, intermediateImageSize ? "512M" }:

vmTools.runInLinuxVM (runCommand "disk-image" {
  nativeBuildInputs = [ kmod parted fatresize dosfstools python3 ];
  preVM = ''
    mkdir scratch
    scratch=$(realpath scratch)
    QEMU_OPTS+=" -virtfs local,path=$scratch,security_model=none,mount_tag=scratch"
  '';
  passthru = {
    inherit src;
  };
} ''
  mkdir /tmp/scratch
  mount -t 9p scratch /tmp/scratch -o trans=virtio,version=9p2000.L,msize=131072
  cd scratch

  modprobe loop

  mkdir /mnt

  # HACK: fatresize segfaults when run on a filesystem that's not on a partition (?)

  img_size=${intermediateImageSize}
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

  cp -r $(find ${src} -mindepth 1 -maxdepth 1) /mnt/

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

  mkdir -p $out
  mv $real_img $out/disk.img
'')
