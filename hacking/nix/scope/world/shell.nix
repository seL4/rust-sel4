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
, seL4RustTargetInfoWithConfig
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

  bindgenEnvVars =
    let
      targets = lib.flatten (
        lib.forEach [ true false ] (cp:
          lib.forEach [ true false ] (minimal:
            seL4RustTargetInfoWithConfig { inherit cp minimal; })));
    in lib.listToAttrs (lib.forEach targets (target: {
      name = "BINDGEN_EXTRA_CLANG_ARGS_${target.name}";
      value = [ "-I${libcDir}/include" ];
    }));
in
mkShell (seL4RustEnvVars // kernelLoaderConfigEnvVars // capdlEnvVars // bindgenEnvVars // {
  # TODO
  RUST_SEL4_TARGET = defaultRustTargetInfo.name;

  RUST_BARE_METAL_TARGET = bareMetalRustTargetInfo.name;

  HOST_CARGO_FLAGS = lib.concatStringsSep " " [
    "-Z" "build-std=core,alloc,compiler_builtins"
    "-Z" "build-std-features=compiler-builtins-mem"
  ];

  HOST_CC = "${buildPackages.stdenv.cc.targetPrefix}gcc";

  LIBCLANG_PATH = libclangPath;

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
    # abbreviation
    export h=$HOST_CARGO_FLAGS
  '';
})
