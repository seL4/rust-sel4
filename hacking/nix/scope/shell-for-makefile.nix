#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mkShell
, python3
, reuse
, cargo-audit
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
      kani
    ];
  };

in
mkShell (apply {
  passthru = {
    inherit apply;
  };
})
