{ lib, stdenv, buildPackages
, writeText, mkShell
, defaultRustToolchain
, defaultRustTargetInfo
, bareMetalRustTargetInfo
, kernel, loaderConfig
, sources
, dummyCapDLSpec, serializeCapDLSpec
}:

let
  loaderConfigJSON = writeText "loader-config.json" (builtins.toJSON loaderConfig);

in
mkShell rec {
  RUST_TARGET_PATH = toString (sources.srcRoot + "/support/targets");

  # TODO
  RUST_SEL4_TARGET = defaultRustTargetInfo.name;
  # RUST_SEL4_TARGET = "aarch64-sel4cp";

  RUST_BARE_METAL_TARGET = bareMetalRustTargetInfo.name;

  HOST_CARGO_FLAGS = lib.concatStringsSep " " [
    "-Z" "build-std=core,alloc,compiler_builtins"
    "-Z" "build-std-features=compiler-builtins-mem"
    # "--target" RUST_SEL4_TARGET
  ];

  LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";

  SEL4_PREFIX = kernel;
  SEL4_LOADER_CONFIG = loaderConfigJSON;
  SEL4_APP = "${kernel}/bin/kernel.elf";

  CAPDL_SPEC_FILE = serializeCapDLSpec { inherit (dummyCapDLSpec.passthru) spec; };
  CAPDL_FILL_DIR = dummyCapDLSpec.passthru.fill;

  hardeningDisable = [ "all" ];

  nativeBuildInputs = [
    defaultRustToolchain
  ];

  depsBuildBuild = [
    buildPackages.stdenv.cc
  ];

  shellHook = ''
    # abbreviation
    export h=$HOST_CARGO_FLAGS
  '';
}
