#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ stdenv, lib
, buildPackages, buildPlatform, hostPlatform
, writeText, writeScript, linkFarm
, fetchRepoProject
, cmake, ninja
, libxml2, dtc, cpio, protobuf
, python3Packages
, qemuForSeL4
, git
, sources
, crateUtils
, defaultRustToolchain
, bareMetalRustTargetInfo
, libclangPath
, vendoredTopLevelLockfile
, buildSysroot
, crates
, pruneLockfile
, topLevelLockfile
, vendorLockfile
, runCommandCC
, seL4Arch
}:

{ mcs ? false
, smp ? false
, virtualization ? false
, rust ? true
, release ? false
, filter ? ".*"
}:

with lib;

let
  rustToolchain = defaultRustToolchain;
  rustTargetInfo = bareMetalRustTargetInfo;

  initBuildArgs =
    let
      bool = v: if v then "TRUE" else "FALSE";
    in [
      "-DCROSS_COMPILER_PREFIX=${stdenv.cc.targetPrefix}"
      "-DKernelSel4Arch=${seL4Arch}"
      "-DMCS=${bool mcs}"
      "-DSMP=${bool smp}"
      "-DSIMULATION=TRUE"
      "-DLibSel4UseRust=${bool rust}"
      "-DLibSel4TestPrinterRegex='${filter}'"
    ] ++ lib.optionals rust [
      "-DHACK_RUST_TARGET=${rustTargetInfo.name}"
      "-DHACK_CARGO_MANIFEST_PATH=${workspace}/Cargo.toml"
      "-DHACK_CARGO_CONFIG=${cargoConfig}"
      "-DHACK_CARGO_NO_BUILD_SYSROOT=TRUE"
      "-DHACK_CARGO_RELEASE=${bool release}"
    ] ++ lib.optionals hostPlatform.isi686 [
      "-DPLATFORM=pc99"
    ] ++ lib.optionals hostPlatform.isRiscV [
      "-DPLATFORM=spike"
      "-DOPENSBI_PLAT_ISA=${hostPlatform.this.gccParams.arch}"
      "-DOPENSBI_PLAT_ABI=${hostPlatform.this.gccParams.abi}"
    ] ++ lib.optionals hostPlatform.isAarch [
      "-DPLATFORM=qemu-arm-virt"
      "-DARM_HYP=${bool virtualization}"
    ] ++ lib.optionals hostPlatform.isAarch32 [
      "-DARM_CPU=cortex-a15"
    ];

  kernelSrc = sources.seL4.rust-sel4test;

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
    release = false; # TODO why?
    inherit rustTargetInfo;
    extraManifest = profiles;
    compilerBuiltinsWeakIntrinsics = true;
  };

  tests = stdenv.mkDerivation {
    name = "sel4test";

    src = fetchRepoProject {
      name = "sel4test";
      manifest = "https://github.com/seL4/sel4test-manifest.git";
      rev = "8bf6fd506a0546866ba5fbd7396f497d5a056f5c";
      sha256 = "sha256-1Gmbksgh2VTUggM6qcawRC9b+g/bwB8tWGfUzCg1A0U=";
    };

    LIBCLANG_PATH = libclangPath;

    depsBuildBuild = lib.optionals (buildPlatform != hostPlatform) [
      buildPackages.stdenv.cc
      # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
      qemuForSeL4
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

      # HACK with this aarch32 toolchain, sizeof(SUCCESS) != sizeof(test_result_t)
      sed -i 's|test_eq(res, SUCCESS);|test_eq((int)res, SUCCESS);|' \
        projects/sel4test/apps/sel4test-tests/src/tests/ipc.c
    '' + lib.optionalString hostPlatform.isx86 ''
      # HACK
      sed -i \
        -e 's|test_scheduler_accuracy,|test_scheduler_accuracy, false \&\&|' \
        -e 's|test_ordering_periodic_threads,|test_ordering_periodic_threads, false \&\&|' \
        projects/sel4test/apps/sel4test-tests/src/tests/scheduler.c
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

    passthru = {
      inherit automate;
    };
  };

  automate =
    let
      py = buildPackages.python3.withPackages (pkgs: [
        pkgs.pexpect
      ]);
    in
      (writeScript "automate" ''
        #!${buildPackages.runtimeShell}
        set -eu

        export PATH=$PATH:${qemuForSeL4.__spliced.buildBuild or qemuForSeL4}/bin

        ${py}/bin/python3 ${./automate.py} ${tests}
      '').overrideAttrs (attrs: {
        passthru = (attrs.passthru or {}) //  {
          testMeta = {
            name = "sel4test-${seL4Arch}";
          };
        };
      });

in
  tests
