{ lib, stdenv, buildPackages
, writeText, mkShell
, defaultRustToolchain
, defaultRustTargetInfo
, bareMetalRustTargetInfo
, sources
, dummyCapDLSpec, serializeCapDLSpec
, seL4RustEnvVars
, worldConfig
, seL4ForBoot
}:

let
  loaderConfigEnvVars = lib.optionalAttrs (worldConfig.loaderConfig != null) {
    SEL4_LOADER_CONFIG = writeText "loader-config.json" (builtins.toJSON worldConfig.loaderConfig);
    SEL4_APP = "${seL4ForBoot}/bin/kernel.elf"; # HACK
  };

in
mkShell (seL4RustEnvVars // loaderConfigEnvVars // {
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
})
