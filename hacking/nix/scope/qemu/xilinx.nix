#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, fetchFromGitHub, fetchFromGitLab
, python3, dtc
, qemuForSeL4
}:

let
  version = "xilinx_v2024.1";

  src = fetchFromGitHub {
    owner = "Xilinx";
    repo = "qemu";
    rev = version;
    hash = "sha256-FKaoTRTftaIId+VkjMqKzcdO48ngQirymH4XLzMm+t8=";
  };

  keycodemapdb = fetchFromGitLab {
    owner = "qemu-project";
    repo = "keycodemapdb";
    rev = "f5772a62ec52591ff6870b7e8ef32482371f22c6";
    hash = "sha256-GbZ5mrUYLXMi0IX4IZzles0Oyc095ij2xAsiLNJwfKQ=";
  };

  berkeley-softfloat-3 = fetchFromGitLab {
    owner = "qemu-project";
    repo = "berkeley-softfloat-3";
    rev = "b64af41c3276f97f0e181920400ee056b9c88037";
    hash = "sha256-Yflpx+mjU8mD5biClNpdmon24EHg4aWBZszbOur5VEA=";
  };

  berkeley-testfloat-3 = fetchFromGitLab {
    owner = "qemu-project";
    repo = "berkeley-testfloat-3";
    rev = "40619cbb3bf32872df8c53cc457039229428a263";
    hash = "sha256-EBz1uYnjehCtJqrSFzERH23N5ELZU3gGM26JnsGFcWg=";
  };

  devicetrees = stdenv.mkDerivation {
    name = "devicetrees";
    src = fetchFromGitHub {
      owner = "Xilinx";
      repo = "qemu-devicetrees";
      rev = version;
      hash = "sha256-FYNo/2XVq/viWIB+yFDwzY5eaosC5omQZ3LyypMu2bM=";
    };
    nativeBuildInputs = [
      python3
      dtc
    ];
    dontConfigure = true;
    dontFixup = true;
    installPhase = ''
      mv LATEST $out
    '';
  };

in

(qemuForSeL4.override {
  hostCpuTargets = [
    "aarch64-softmmu"
  ];
}).overrideAttrs (finalAttrs: previousAttrs: {
  inherit version src;

  postPatch = (previousAttrs.postPatch or "") + ''
    pushd subprojects
      ln -sf ${keycodemapdb} keycodemapdb

      d=berkeley-softfloat-3
      cp -r --no-preserve=owner,mode ${berkeley-softfloat-3} $d
      cp -r packagefiles/$d/* $d

      d=berkeley-testfloat-3
      cp -r --no-preserve=owner,mode ${berkeley-testfloat-3} $d
      cp -r packagefiles/$d/* $d
    popd
  '';

  passthru = (previousAttrs.passthru or {}) // {
    inherit devicetrees;
  };
})
