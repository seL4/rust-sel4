{ lib, stdenv, buildPackages
, writeText, emptyFile
, mkShell
, defaultRustToolchain
, defaultRustTargetInfo
, bareMetalRustTargetInfo
, libclangPath
, sources
, dummyCapDLSpec, serializeCapDLSpec
, seL4RustEnvVars
, worldConfig
, seL4ForBoot
, crateUtils

, hostPlatform
, cmake
, perl
, python3Packages
}:

let
  kernelLoaderConfigEnvVars = lib.optionalAttrs (!worldConfig.isCorePlatform && worldConfig.kernelLoaderConfig != null) {
    SEL4_KERNEL_LOADER_CONFIG = writeText "loader-config.json" (builtins.toJSON worldConfig.kernelLoaderConfig);
  };

  capdlEnvVars = lib.optionalAttrs (!worldConfig.isCorePlatform) {
    CAPDL_SPEC_FILE = serializeCapDLSpec { inherit (dummyCapDLSpec.passthru) spec; };
    CAPDL_FILL_DIR = dummyCapDLSpec.passthru.fill;
  };

  libcDir = "${stdenv.cc.libc}/${hostPlatform.config}";
in
mkShell (seL4RustEnvVars // kernelLoaderConfigEnvVars // capdlEnvVars // {
  # TODO
  RUST_SEL4_TARGET = defaultRustTargetInfo.name;

  RUST_BARE_METAL_TARGET = bareMetalRustTargetInfo.name;

  HOST_CARGO_FLAGS = lib.concatStringsSep " " [
    "-Z" "build-std=core,alloc,compiler_builtins"
    "-Z" "build-std-features=compiler-builtins-mem"
  ];

  LIBCLANG_PATH = libclangPath;

  hardeningDisable = [ "all" ];

  "BINDGEN_EXTRA_CLANG_ARGS_${defaultRustTargetInfo.name}" = [ "-I${libcDir}/include" ];

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
    # abbreviation
    export h=$HOST_CARGO_FLAGS
  '';
})
