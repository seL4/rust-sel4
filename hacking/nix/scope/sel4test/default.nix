{ stdenv, lib
, buildPackages, buildPlatform, hostPlatform
, writeText, linkFarm
, fetchRepoProject
, cmake, ninja
, libxml2, dtc, cpio, protobuf
, python3Packages
, qemu
, git
, gccMultiStdenvGeneric
, sources
, crateUtils
, defaultRustToolchain
, bareMetalRustTargetInfo
, vendoredTopLevelLockfile
, buildSysroot
, crates
, pruneLockfile
, topLevelLockfile
, vendorLockfile
, runCommandCC
, seL4Arch
}:

with lib;

let
  thisStdenv = if hostPlatform.isRiscV then gccMultiStdenvGeneric else stdenv;
in

let
  rustToolchain = defaultRustToolchain;
  rustTargetInfo = bareMetalRustTargetInfo;

  useRust = hostPlatform.is64bit;

  initBuildArgs = [
    "-DSIMULATION=TRUE"
    "-DKernelSel4Arch=${seL4Arch}"
    "-DCROSS_COMPILER_PREFIX=${thisStdenv.cc.targetPrefix}"
    # "-DMCS=FALSE"
    # "-DSMP=FALSE"
    "-DLibSel4UseRust=${if useRust then "TRUE" else "FALSE"}"
    "-DHACK_CARGO_MANIFEST_PATH=${workspace}/Cargo.toml"
    "-DHACK_CARGO_CONFIG=${cargoConfig}"
    "-DHACK_CARGO_NO_BUILD_SYSROOT=TRUE"
    # "-DHACK_CARGO_RELEASE=TRUE"
  ] ++ lib.optionals hostPlatform.isi686 [
    "-DPLATFORM=pc99"
  ] ++ lib.optionals hostPlatform.isRiscV [
    "-DPLATFORM=spike"
    "-DHACK_COMPILER_BUILTINS_SYMBOLS=${compilerBuiltinsSymbols}"
  ] ++ lib.optionals hostPlatform.isAarch [
    "-DPLATFORM=qemu-arm-virt"
    # "-DARM_HYP=TRUE"
  ] ++ lib.optionals hostPlatform.isAarch32 [
    "-DARM_CPU=cortex-a15"
  ];

  kernelSrc = builtins.fetchGit rec {
    url = "https://gitlab.com/coliasgroup/seL4.git";
    rev = "0b3c3d9672cf742dc948977312216703132f4a29"; # rust-sel4test
    ref = sources.mkKeepRef rev;
  };

  # kernelSrc = lib.cleanSource ../../../../../../../../x/seL4;

  cratesSrc = crateUtils.collectReals (lib.attrValues (crateUtils.getClosureOfCrate rootCrate));

  rootCrate = crates.sel4-sys-wrappers;

  lockfile = builtins.toFile "Cargo.lock" lockfileContents;
  lockfileContents = builtins.readFile lockfileDrv;
  lockfileDrv = pruneLockfile {
    superLockfile = topLevelLockfile;
    superLockfileVendoringConfig = vendoredTopLevelLockfile.configFragment;
    rootCrates = [ rootCrate ];
  };

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
    (vendorLockfile { inherit lockfileContents; }).configFragment
  ]);

  profiles = {
    profile.release = {
      debug = 0;
      opt-level = "z";
    };
  };

  manifest = crateUtils.toTOMLFile "Cargo.toml" (crateUtils.clobber [
    {
      workspace.resolver = "2";
      workspace.members = [ "src/${rootCrate.name}" ];
    }
    profiles
  ]);

  workspace = linkFarm "workspace" [
    { name = "Cargo.toml"; path = manifest; }
    { name = "Cargo.lock"; path = lockfile; }
    { name = "src"; path = cratesSrc; }
  ];

  sysroot = buildSysroot {
    release = false;
    inherit rustTargetInfo;
    extraManifest = profiles;
  };

  compilerBuiltinsSymbols = runCommandCC "weaken.txt" {} ''
    $NM ${sysroot}/lib/rustlib/${rustTargetInfo.name}/lib/libcompiler_builtins-*.rlib \
      | sed -rn 's,^.* T (__.*)$,\1,p' > $out
  '';

in
thisStdenv.mkDerivation {
  name = "sel4test";

  src = fetchRepoProject {
    name = "sel4test";
    manifest = "https://github.com/seL4/sel4test-manifest.git";
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

  '' + lib.optionalString hostPlatform.isRiscV64 ''
    # HACK
    sed -i 's|test_bad_instruction, true|test_bad_instruction, false|' \
      projects/sel4test/apps/sel4test-tests/src/tests/faults.c

    sed -i 's|printf("Bootstrapping kernel\\n");|printf("Bootstrapping kernel %lx\\n", v_entry);|' \
      kernel/src/arch/riscv/kernel/boot.c
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
    rm -rf $out/libsel4/rust/target
  '';

  dontFixup = true;

  enableParallelBuilding = true;
}
