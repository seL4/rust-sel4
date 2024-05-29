#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, runCommand
, crateUtils
, defaultRustToolchain
}:

{ rustToolchain ? defaultRustToolchain
, rootCrates
, vendoredSuperLockfile
, extraManifest ? {}, extraConfig ? {}
}:

let
  crates = lib.attrValues (crateUtils.getClosureOfCrates rootCrates);

  manifest = crateUtils.toTOMLFile "Cargo.toml" (crateUtils.clobber [
    {
      workspace.resolver = "2";
      workspace.members = map (crate: "src/${crate.name}") rootCrates;
    }
    extraManifest
  ]);

  src = crateUtils.collectDummies crates;

  config = crateUtils.toTOMLFile "config" (crateUtils.clobber [
    {
      unstable.unstable-options = true;
    }
    vendoredSuperLockfile.configFragment
    extraConfig
  ]);

in
runCommand "Cargo.lock" {
  nativeBuildInputs = [
    rustToolchain
  ];
} ''
  ln -s ${manifest} Cargo.toml
  ln -s ${src} src
  cp --no-preserve=owner,mode ${vendoredSuperLockfile.lockfile} Cargo.lock
  cargo --config ${config} update -wq
  mv Cargo.lock $out
''
