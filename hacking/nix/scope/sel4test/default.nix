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
, runCommandCC
, seL4Arch
}:

{ mcs ? false
, smp ? false
, virtualization ? false
, filter ? ".*"
}:

with lib;

let
  initBuildArgs =
    let
      bool = v: if v then "TRUE" else "FALSE";
    in [
      "-DCROSS_COMPILER_PREFIX=${stdenv.cc.targetPrefix}"
      "-DKernelSel4Arch=${seL4Arch}"
      "-DMCS=${bool mcs}"
      "-DSMP=${bool smp}"
      "-DSIMULATION=TRUE"
      "-DLibSel4TestPrinterRegex='${filter}'"
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

  kernelSrc = sources.seL4.rust;

  tests = stdenv.mkDerivation {
    name = "sel4test";

    src = fetchRepoProject {
      name = "sel4test";
      manifest = "https://github.com/seL4/sel4test-manifest.git";
      rev = "8bf6fd506a0546866ba5fbd7396f497d5a056f5c";
      sha256 = "sha256-1Gmbksgh2VTUggM6qcawRC9b+g/bwB8tWGfUzCg1A0U=";
    };

    depsBuildBuild = lib.optionals (buildPlatform != hostPlatform) [
      buildPackages.stdenv.cc
      # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
      qemuForSeL4
    ];

    nativeBuildInputs = [
      cmake ninja
      libxml2 dtc cpio protobuf
      git
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
