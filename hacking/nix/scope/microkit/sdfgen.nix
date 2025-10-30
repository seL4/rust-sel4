#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, linkFarm
, fetchzip
, python312Packages
, zig-overlay
, sources
}:

let
  deps = linkFarm "zig-packages" [
    {
      name = "dtb-0.0.0-gULdmfUKAgB8JOQL-RUq6oDDJIuB8msl-m2ZfnGx_W0W";
      path = fetchzip {
        url = "https://github.com/Ivan-Velickovic/dtb.zig/archive/fc940d8ebefebe6f27713ebc92fda1ee7fe342c7.tar.gz";
        hash = "sha256-vUfOPtGLWl2gqkmL6v/KtrXvMcihVyXTLZTBQG8ntyI=";
      };
    }
  ];
in
with python312Packages;
buildPythonPackage rec {
  name = "sdfgen";
  src = sources.sdfgen;

  build-system = [ setuptools ];

  pythonImportsCheck = [ "sdfgen" ];

  postPatch = ''
    export ZIG_LOCAL_CACHE_DIR=$(mktemp -d)
    export ZIG_GLOBAL_CACHE_DIR=$(mktemp -d)
    ln -s ${deps} $ZIG_GLOBAL_CACHE_DIR/p
  '';

  nativeBuildInputs = [ zig-overlay."0.14.0" ];
}
