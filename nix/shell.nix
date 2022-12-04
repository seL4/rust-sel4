{ lib, stdenv, writeText
, buildPackages
, mkShell
, llvmPackages
, rustup
, git
, cacert
, qemu
}:

{ kernel, loaderConfig
, rustSeL4Target, rustBareMetalTarget
}:

let
  loaderConfigJSON = writeText "loader-config.json" (builtins.toJSON loaderConfig);

in
mkShell {

  RUST_TARGET_PATH = toString ../support/targets; # absolute path

  RUST_SEL4_TARGET = rustSeL4Target;
  RUST_BARE_METAL_TARGET = rustBareMetalTarget;

  # for bindgen
  LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";

  # for local crates
  SEL4_PREFIX = kernel;
  SEL4_LOADER_CONFIG = loaderConfigJSON;

  hardeningDisable = [ "all" ];

  nativeBuildInputs = [
    rustup
    git
    cacert
  ];

  depsBuildBuild = [
    buildPackages.stdenv.cc
    qemu
  ];
}
