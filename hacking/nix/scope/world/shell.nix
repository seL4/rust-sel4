#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPackages
, writeText, emptyFile
, mkShell
, defaultRustToolchain
, defaultRustTargetTriple
, bareMetalRustTargetTriple
, libclangPath
, sources
, seL4RustEnvVars
, allCustomRustTargetTripleNames
, worldConfig
, seL4ForBoot
, crateUtils

, hostPlatform
, cmake
, perl
, python3Packages
}:

let
  kernelLoaderConfigEnvVars = lib.optionalAttrs (!worldConfig.isMicrokit && worldConfig.kernelLoaderConfig != null) {
    SEL4_KERNEL_LOADER_CONFIG = writeText "loader-config.json" (builtins.toJSON worldConfig.kernelLoaderConfig);
  };

  libcDir = "${stdenv.cc.libc}/${hostPlatform.config}";

  bindgenEnvVars =
    lib.listToAttrs (lib.forEach allCustomRustTargetTripleNames (targetName: {
      name = "BINDGEN_EXTRA_CLANG_ARGS_${targetName}";
      value = [ "-I${libcDir}/include" ];
    }));

  miscEnvVars = {
    CHILD_ELF = emptyFile;
  };

  buildStdFlags = std: lib.concatStringsSep " " [
    "-Z" "build-std=core,alloc,compiler_builtins${lib.optionalString std ",std"}"
    "-Z" "build-std-features=compiler-builtins-mem"
  ];

in
mkShell (seL4RustEnvVars // kernelLoaderConfigEnvVars // bindgenEnvVars // miscEnvVars // {
  # TODO
  RUST_SEL4_TARGET = defaultRustTargetTriple.name;

  RUST_BARE_METAL_TARGET = bareMetalRustTargetTriple.name;

  HOST_CARGO_FLAGS = buildStdFlags false;

  HOST_CARGO_FLAGS_STD = buildStdFlags true;

  HOST_CC = "${buildPackages.stdenv.cc.targetPrefix}gcc";

  LIBCLANG_PATH = libclangPath;

  SDDF_INCLUDE_DIRS =
    let
      d = sources.sddf;
    in
      lib.concatStringsSep ":" [
        "${d}/include"
        "${d}/include/microkit"
      ];

  LIONSOS_INCLUDE_DIRS =
    let
      d = sources.lionsos;
    in
      lib.concatStringsSep ":" [
        "${d}/include"
      ];

  hardeningDisable = [ "all" ];

  nativeBuildInputs = [
    defaultRustToolchain
    cmake
    perl
    python3Packages.jsonschema
    python3Packages.jinja2
  ];

  depsBuildBuild = [
    buildPackages.stdenv.cc
  ];

  shellHook = ''
    export h=$HOST_CARGO_FLAGS
    export hs=$HOST_CARGO_FLAGS_STD
    export t="--target $RUST_SEL4_TARGET"
    export tb="--target $RUST_BARE_METAL_TARGET"
  '';
})
