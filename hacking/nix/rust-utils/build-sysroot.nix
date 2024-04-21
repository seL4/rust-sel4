#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPlatform, hostPlatform, buildPackages
, runCommand, linkFarm
, vendorLockfile, crateUtils, symlinkToRegularFile
, defaultRustToolchain, defaultRustTargetInfo
, rustToolchain ? defaultRustToolchain
}:

let
  sysrootLockfile = symlinkToRegularFile "Cargo.lock" "${rustToolchain}/lib/rustlib/src/rust/Cargo.lock";

  # NOTE
  # There is one thunk per package set.
  # Consolidating further would improve eval perf.
  vendoredCrates = vendorLockfile { lockfile = sysrootLockfile; };
in

{ release ? true
, profile ? if release then "release" else null
, extraManifest ? {}
, extraConfig ? {}
, rustTargetInfo ? defaultRustTargetInfo
, alloc ? true
, compilerBuiltinsMem ? true
}:

let
  workspace = linkFarm "workspace" [
    { name = "Cargo.toml"; path = manifest; }
    { name = "Cargo.lock"; path = lockfile; }
  ];

  package = {
    name = "dummy";
    version = "0.0.0";
  };

  manifest = crateUtils.toTOMLFile "Cargo.toml" (crateUtils.clobber [
    {
      inherit package;
      lib.path = crateUtils.dummyLibInSrc;
    }
    extraManifest
  ]);

  lockfile = crateUtils.toTOMLFile "Cargo.lock" {
    package = [
      package
    ];
  };

  config = crateUtils.toTOMLFile "config" (crateUtils.clobber [
    # baseConfig # TODO will trigger rebuild
    {
      target = {
        "${rustTargetInfo.name}" = {
          rustflags = [
            # "-C" "force-unwind-tables=yes" # TODO compare with "requires-uwtable" in target.json
            "-C" "embed-bitcode=yes"
            "--sysroot" "/dev/null"
          ];
        };
      };
    }
    vendoredCrates.configFragment
    extraConfig
  ]);

  crates = lib.concatStringsSep "," ([
    "core"
    "compiler_builtins"
  ] ++ lib.optionals alloc [
    "alloc"
  ]);

  features = lib.concatStringsSep "," (lib.optionals compilerBuiltinsMem [
    "compiler-builtins-mem"
  ]);

in
runCommand "sysroot" {
  depsBuildBuild = [ buildPackages.stdenv.cc ];
  nativeBuildInputs = [ rustToolchain ];

  RUST_TARGET_PATH = rustTargetInfo.path;
} ''
  cargo build \
    -Z unstable-options \
    --offline \
    --frozen \
    --config ${config} \
    ${lib.optionalString (profile != null) "--profile ${profile}"} \
    --target ${rustTargetInfo.name} \
    -Z build-std=${crates} \
    -Z build-std-features=${features} \
    --manifest-path ${workspace}/Cargo.toml \
    --target-dir $(pwd)/target

  d=$out/lib/rustlib/${rustTargetInfo.name}/lib
  mkdir -p $d
  mv target/${rustTargetInfo.name}/*/deps/* $d
''

# TODO
# rel=lib/rustlib/${buildPlatform.config}/bin
# d=$out/$rel
# mkdir -p $d
# ln -s ${rustToolchain}/$rel/* $d

# NOTE "-Z avoid-dev-deps" for deps of std
