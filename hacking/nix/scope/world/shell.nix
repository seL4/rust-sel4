{ lib, stdenv, buildPackages
, writeText, mkShell
, defaultRustToolchain
, defaultRustTargetName
, defaultRustTargetPath
, bareMetalRustTargetName
, kernel, loaderConfig
, srcRoot
}:

let
  loaderConfigJSON = writeText "loader-config.json" (builtins.toJSON loaderConfig);

in
mkShell rec {
  RUST_TARGET_PATH = toString (srcRoot + "/support/targets");

  # TODO
  RUST_SEL4_TARGET = defaultRustTargetName;
  # RUST_SEL4_TARGET = "aarch64-sel4cp";

  RUST_BARE_METAL_TARGET = bareMetalRustTargetName;

  HOST_CARGO_FLAGS = lib.concatStringsSep " " [
    "-Z" "build-std=core,alloc,compiler_builtins"
    "-Z" "build-std-features=compiler-builtins-mem"
    "--target" RUST_SEL4_TARGET
  ];

  LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";

  SEL4_PREFIX = kernel;
  SEL4_LOADER_CONFIG = loaderConfigJSON;

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
