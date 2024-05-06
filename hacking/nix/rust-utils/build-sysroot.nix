#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPlatform, hostPlatform, buildPackages
, runCommand, runCommandCC, linkFarm
, fetchurl
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
, compilerBuiltinsC ? true
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
  ] ++ lib.optionals compilerBuiltinsC [
    "compiler-builtins-c"
  ]);

  compilerRTSource = let
    v = "18.0-2024-02-13";
    name = "compiler-rt";
    llvmSourceTarball = fetchurl {
      name = "llvm-project.tar.gz";
      url = "https://github.com/rust-lang/llvm-project/archive/rustc/${v}.tar.gz";
      sha256 = "sha256-fMc84lCWfNy0Xiq1X7nrT53MQPlfRqGEb4qBAmqehAA=";
    };
  in
    runCommand name {} ''
      tar xzf ${llvmSourceTarball} --strip-components 1 llvm-project-rustc-${v}/${name}
      mv ${name} $out
    '';

in
(if compilerBuiltinsC then runCommandCC else runCommand) "sysroot" ({
  depsBuildBuild = [ buildPackages.stdenv.cc ];
  nativeBuildInputs = [ rustToolchain ];
  RUST_TARGET_PATH = rustTargetInfo.path;
} // lib.optionalAttrs compilerBuiltinsC {
  "CC_${rustTargetInfo.name}" = "${stdenv.cc.targetPrefix}gcc";
  RUST_COMPILER_RT_ROOT = compilerRTSource;
}) ''
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
