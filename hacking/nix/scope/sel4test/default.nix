{ stdenv, lib
, buildPackages, buildPlatform, hostPlatform
, writeText
, fetchRepoProject
, cmake, ninja
, libxml2, dtc, cpio, protobuf
, python3Packages
, qemu
, git
, gccMultiStdenvGeneric
, mkKeepRef
, crateUtils
, defaultRustToolchain
, bareMetalRustTargetInfo
, vendoredTopLevelLockfile
, buildSysroot
}:

with lib;

let
  thisStdenv = if hostPlatform.isRiscV64 then gccMultiStdenvGeneric else stdenv;
in

let
  rustToolchain = defaultRustToolchain;
  rustTargetInfo = bareMetalRustTargetInfo;

  initBuildArgs = [
    "-DSIMULATION=TRUE"
    "-DKernelSel4Arch=${hostPlatform.parsed.cpu.name}"
    "-DCROSS_COMPILER_PREFIX=${thisStdenv.cc.targetPrefix}"
    # "-DMCS=FALSE"
    # "-DSMP=FALSE"
    "-DLibSel4UseRust=TRUE"
    # "-DLibSel4UseRust=FALSE"
    "-DHACK_RUST_SEL4_SOURCE_DIR=${rustSourceDir}"
    "-DHACK_CARGO_CONFIG=${cargoConfig}"
    "-DHACK_RUST_NO_BUILD_SYSROOT=TRUE"
  ] ++ lib.optionals hostPlatform.isRiscV64 [
    "-DPLATFORM=spike"
  ] ++ lib.optionals hostPlatform.isAarch64 [
    "-DPLATFORM=qemu-arm-virt"
    # "-DARM_HYP=TRUE"
  ];

  kernelSrc = builtins.fetchGit rec {
    url = "https://gitlab.com/coliasgroup/seL4.git";
    rev = "4d82588c2111e162d996455c2d2b6d9253661d1c";
    ref = mkKeepRef rev;
  };

  # kernelSrc = lib.cleanSource ../../../../../../../../x/seL4;

  cargoConfig = crateUtils.toTOMLFile "config" (crateUtils.clobber [
    (crateUtils.baseConfig {
      inherit rustToolchain;
      rustTargetName = rustTargetInfo.name;
    })
    {
      target.${rustTargetInfo.name}.rustflags = [
        "--sysroot" sysroot
      ];
    }
    vendoredTopLevelLockfile.configFragment
  ]);

  rustSourceDir = lib.cleanSourceWith {
    src = lib.cleanSource ../../../..;
    filter = name: type:
    let baseName = baseNameOf (toString name); in !(
      false
        || baseName == "hacking"
    );
  };

  sysroot = buildSysroot {
    release = false;
    inherit rustTargetInfo;
    # extraManifest = profiles;
  };

in
thisStdenv.mkDerivation {
  name = "sel4test";

  src = fetchRepoProject {
    name = "sel4test";
    manifest = "https://github.com/seL4/sel4test-manifest.git";
    # rev = "499db1117efce7ce8361e71193de5098c71f9af8";
    # sha256 = "sha256-+/a0ulPG/WotdBD4ORgIffm8Q1JS+ofGfN6s0Bm/onU=";
    rev = "cfc1195ba8fd0de1a0e179aef1314b8f402ff74c";
    sha256 = "sha256-JspN1A/w5XIV+XCj5/oj7NABsKXVdr+UZOTJWvfJPUY=";
  };

  LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";

  depsBuildBuild = lib.optionals (buildPlatform != hostPlatform) [
    buildPackages.stdenv.cc
    # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
    qemu
  ];

  nativeBuildInputs = [
    cmake ninja
    libxml2 dtc cpio protobuf
    git
    defaultRustToolchain
  ] ++ (with python3Packages; [
    aenum plyplus pyelftools simpleeval
    sel4-deps
    buildPackages.python3Packages.protobuf
  ]);

  hardeningDisable = [ "all" ];

  postPatch = ''
    rm -r kernel
    cp -r --no-preserve=ownership ${kernelSrc} kernel
    chmod -R +w kernel

    # patchShebangs can't handle env -S
    rm kernel/configs/*_verified.cmake
    rm tools/seL4/cmake-tool/helpers/cmakerepl

    patchShebangs --build .

    # HACK
    rm projects/musllibc/.git

    # HACK
    sed -i 's,--enable-static,--enable-static --disable-visibility,' \
      projects/musllibc/Makefile

    # HACK
    sed -i 's,tput reset,true,' \
      tools/seL4/cmake-tool/simulate_scripts/simulate.py
  '';

  configurePhase = ''
    mkdir build
    cd build
    ../init-build.sh ${lib.concatStringsSep " " initBuildArgs}
  '';

  buildPhase = ''
    ninja
  '';

  installPhase = ''
    cd ..
    mv build $out
  '';

  dontFixup = true;

  enableParallelBuilding = true;
}
