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
      name = "dtb-0.0.0-gULdmT8JAgAO49xxrRGA_0_0v4nPL7D91Ev2x7NnNbmy";
      path = fetchzip {
        url = "https://github.com/Ivan-Velickovic/dtb.zig/archive/13d4cc60806f4655043d00df50d4225737b268d4.tar.gz";
        hash = "sha256-V5L3/B7mQ6OubTyIUbHDxGJSm+pbIYcoyJcOAReMhTk=";
      };
    }
    {
      name = "sddf-0.0.0-6aJ67hfnZgCIb73S-PWM-oHDp0RadrbTQqB2Cc7wDlln";
      path = fetchzip {
        url = "https://github.com/au-ts/sddf/archive/e8341acea643c818e59033812accdc531fb82201.tar.gz";
        hash = "sha256-0PEHzD5mkCkRK3LJ8exkJi3rgzr3ZS8UzE9V3OonA1g=";
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

  nativeBuildInputs = [ zig-overlay."0.15.1" ];
}
