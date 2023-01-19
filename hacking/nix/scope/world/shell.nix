{ lib, stdenv, buildPackages
, writeText, mkShell
, defaultRustToolchain
, defaultRustTargetName
, defaultRustTargetPath
, bareMetalRustTargetName
, kernel, loaderConfig
}:

let
  loaderConfigJSON = writeText "loader-config.json" (builtins.toJSON loaderConfig);

in
mkShell {
  RUST_TARGET_PATH = defaultRustTargetPath;
  RUST_SEL4_TARGET = defaultRustTargetName;
  RUST_BARE_METAL_TARGET = bareMetalRustTargetName;

  HOST_CARGO_FLAGS = lib.concatStringsSep " " [
	  "-Z" "build-std=core,alloc,compiler_builtins"
	  "-Z" "build-std-features=compiler-builtins-mem"
    "--target" defaultRustTargetName
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
