#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, hostPlatform
, mkShell
, python3
, reuse
, cargo-audit
, lychee
, kani
}:

let
  # HACK for composability
  apply = attrs: attrs // {
    IN_NIX_SHELL_FOR_MAKEFILE = 1;

    hardeningDisable = [ "all" ];

    nativeBuildInputs = (attrs.nativeBuildInputs or []) ++ [
      python3
      reuse
      cargo-audit
      lychee
    ] ++ lib.optionals hostPlatform.isx86_64 [
      kani
    ];
  };

in
mkShell (apply {
  passthru = {
    inherit apply;
  };
})
