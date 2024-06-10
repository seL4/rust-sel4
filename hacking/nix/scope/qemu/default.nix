#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPackages
, fetchurl, fetchpatch, fetchFromGitHub, fetchFromGitLab
, python3Packages
, pkg-config, ninja, meson, perl
, zlib, lzo, glib
, bison, flex, dtc
, pixman, vde2
, texinfo
, snappy, libaio, libtasn1, gnutls, nettle, curl
, attr, libcap, libcap_ng, libslirp

, hostCpuTargets ? [
    "arm-softmmu"
    "aarch64-softmmu"
    "riscv32-softmmu"
    "riscv64-softmmu"
    "i386-softmmu"
    "x86_64-softmmu"
  ]
}:

stdenv.mkDerivation (finalAttrs: with finalAttrs; {
  pname = "qemu";
  version = "9.0.0";

  src = fetchurl {
    url = "https://download.qemu.org/qemu-${version}.tar.xz";
    hash = "sha256-MnCKxmww2MiSYz6paMdxwcdtWX1w3erSGg0izPOG2mk=";
  };

  depsBuildBuild = [
    buildPackages.stdenv.cc
  ];

  nativeBuildInputs = [
    pkg-config meson ninja
    bison flex dtc
    perl

    # Don't change this to python3 and python3.pkgs.*, breaks cross-compilation
    python3Packages.python
    python3Packages.sphinx
    python3Packages.sphinx-rtd-theme
  ];

  buildInputs = [
    dtc zlib lzo glib pixman vde2 texinfo
    snappy libtasn1 gnutls nettle curl libslirp
    libaio libcap_ng libcap attr
  ];

  dontUseMesonConfigure = true; # meson's configurePhase isn't compatible with qemu build

  patches = [
    # nspin/arm-virt-sp804
    (fetchurl {
      url = "https://github.com/coliasgroup/qemu/commit/7994b0d17da7dbf1cf2da3e6555914e23559b23e.patch";
      sha256 = "sha256-o5Z1LYF6pwqBGrP4AYOIXmhSg75w7mIRuxvj2ZCO+HY=";
    })
    # nspin/opensbi-fw-payload-use-elf-entry-point
    (fetchurl {
      url = "https://github.com/coliasgroup/qemu/commit/db69d0a7dc0af9d8130328347fdd81ab5fa9e352.patch";
      sha256 = "sha256-12uGZRO6T1uWYvblAx5/FdRsuZZ1B1iWT9ZxpN3Qga0=";
    })
  ];

  postPatch = ''
    # Otherwise tries to ensure /var/run exists.
    sed -i "/install_emptydir(get_option('localstatedir') \/ 'run')/d" \
        qga/meson.build
  '';

  preConfigure = ''
    unset CPP # intereferes with dependency calculation
    # this script isn't marked as executable b/c it's indirectly used by meson. Needed to patch its shebang
    chmod +x ./scripts/shaderinclude.py
    patchShebangs .
    # avoid conflicts with libc++ include for <version>
    mv VERSION QEMU_VERSION
    substituteInPlace configure \
      --replace '$source_path/VERSION' '$source_path/QEMU_VERSION'
    substituteInPlace meson.build \
      --replace "'VERSION'" "'QEMU_VERSION'"
  '';

  configureFlags = [
    "--localstatedir=/var"
    "--sysconfdir=/etc"
    "--enable-linux-aio"
    "--enable-slirp"
    "--cross-prefix=${stdenv.cc.targetPrefix}"
    "--target-list=${lib.concatStringsSep "," hostCpuTargets}"
  ];

  preBuild = ''
    cd build
  '';
})
