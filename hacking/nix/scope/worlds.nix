{ lib, pkgsBuildBuild
, cmakeConfigHelpers
, mkWorld
, platUtils
, opensbi
}:

with cmakeConfigHelpers;

let
  kernelConfigCommon = {
    KernelVerificationBuild = off;
    KernelRootCNodeSizeBits = mkString "14"; # For backtrace test with embedded debug info
  };

  kernelLoaderConfig = {};

  corePlatformConfig = {};

in rec {
  aarch64 =
    let
    in rec {
      default = qemu-arm-virt.default;
      qemu-arm-virt =
        let
          mk =
            { smp ? false
            , mcs ? false
            , el2 ? true
            , hypervisor ? false
            , cpu ? "cortex-a57"
            , isCorePlatform ? false
            , mkSeL4KernelWithPayloadArgs ? loader: [ "-kernel" loader ]
            }:
            let
              numCores = if smp then "2" else "1";
            in
              mkWorld {
                inherit kernelLoaderConfig corePlatformConfig;
                inherit isCorePlatform;
                kernelConfig = kernelConfigCommon // {
                  ARM_CPU = mkString cpu;
                  KernelArch = mkString "arm";
                  KernelSel4Arch = mkString "aarch64";
                  KernelPlatform = mkString "qemu-arm-virt";
                  KernelMaxNumNodes = mkString numCores;
                  KernelIsMCS = fromBool mcs;

                  # benchmarking config
                  # KernelDebugBuild = off;
                  # KernelBenchmarks = mkString "track_utilisation";
                  # KernelArmExportPMUUser = on;
                  # KernelSignalFastpath = on;

                } // lib.optionalAttrs hypervisor {
                  KernelArmHypervisorSupport = on;
                };
                canSimulate = true;
                mkInstanceForPlatform = platUtils.qemu.mkMkInstanceForPlatform {
                  mkQemuCmd = loader: [
                    # NOTE
                    # virtualization=on even when hypervisor to test loader dropping exception level
                    "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-aarch64"
                      "-machine" "virt${lib.optionalString el2 ",virtualization=on"}"
                      "-cpu" cpu "-smp" numCores "-m" "1024"
                      "-nographic"
                      "-serial" "mon:stdio"
                  ] ++ mkSeL4KernelWithPayloadArgs loader;
                };
              };
        in rec {
          default = el2;
          el1 = mk { smp = true; };
          el2 = mk { smp = true; hypervisor = true; };
          el2MCS = mk { smp = true; hypervisor = true; mcs = true; };
          sel4cp = mk {
            el2 = false;
            mcs = true;
            isCorePlatform = true;
            cpu = "cortex-a53";
            mkSeL4KernelWithPayloadArgs = loader: [ "-device" "loader,file=${loader},addr=0x70000000,cpu-num=0" ];
          };
        };

      bcm2711 = mkWorld {
        inherit kernelLoaderConfig;
        kernelConfig = kernelConfigCommon // {
          RPI4_MEMORY = mkString "4096";
          KernelArch = mkString "arm";
          KernelSel4Arch = mkString "aarch64";
          KernelPlatform = mkString "bcm2711";
          KernelArmHypervisorSupport = on;
          KernelMaxNumNodes = mkString "4";
        };
        mkInstanceForPlatform = platUtils.rpi4.mkInstanceForPlatform;
      };
    };

  aarch32 =
    let
    in rec {
      default = qemu-arm-virt.default;
      qemu-arm-virt =
        let
          mk = { smp ? false, mcs ? false, cpu ? "cortex-a15" }:
            let
              numCores = if smp then "2" else "1";
            in
              mkWorld {
                inherit kernelLoaderConfig;
                isCorePlatform = false;
                kernelConfig = kernelConfigCommon // {
                  ARM_CPU = mkString cpu;
                  KernelArch = mkString "arm";
                  KernelSel4Arch = mkString "aarch32";
                  KernelPlatform = mkString "qemu-arm-virt";
                  KernelMaxNumNodes = mkString numCores;
                  KernelIsMCS = fromBool mcs;
                };
                mkInstanceForPlatform = platUtils.qemu.mkMkInstanceForPlatform {
                  mkQemuCmd = loader: [
                    "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-aarch32"
                      "-machine" "virt"
                      "-cpu" cpu "-smp" numCores "-m" "1024"
                      "-nographic"
                      "-serial" "mon:stdio"
                      "-kernel" loader
                  ];
                };
              };
        in rec {
          default = legacy;
          legacy = mk {};
          mcs = mk { mcs = true; };
        };
    };

  riscv64 =
    let
    in rec {
      default = qemu-riscv-virt.default;

      qemu-riscv-virt =
        let
          mk =
            { mcs ? false
            , smp ? false
            }:
            let
              numCores = if smp then "2" else "1";
              qemuMemory = "3072";
            in
              mkWorld {
                inherit kernelLoaderConfig corePlatformConfig;
                kernelConfig = kernelConfigCommon // {
                  QEMU_MEMORY = mkString qemuMemory;
                  KernelArch = mkString "riscv";
                  KernelSel4Arch = mkString "riscv64";
                  KernelPlatform = mkString "qemu-riscv-virt";
                  KernelMaxNumNodes = mkString numCores;
                  KernelIsMCS = fromBool mcs;
                };
                canSimulate = true;
                mkInstanceForPlatform = platUtils.qemu.mkMkInstanceForPlatform {
                  mkQemuCmd = loader: [
                    "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-riscv64"
                      "-machine" "virt"
                      "-cpu" "rv64" "-smp" numCores "-m" qemuMemory
                      "-nographic"
                      "-serial" "mon:stdio"
                      "-kernel" loader
                  ];
                };
              };
        in rec {
          default = legacy.smp;
          mcs = {
            smp = mk { mcs = true; smp = true; };
            nosmp = mk { mcs = true; smp = false; };
          };
          legacy = {
            smp = mk { mcs = false; smp = true; };
            nosmp = mk { mcs = false; smp = false; };
          };
        };

      spike = mkWorld {
        inherit kernelLoaderConfig;
        kernelConfig = kernelConfigCommon // {
          KernelArch = mkString "riscv";
          KernelSel4Arch = mkString "riscv64";
          KernelPlatform = mkString "spike";
        };
        canSimulate = true;
        mkInstanceForPlatform = platUtils.qemu.mkMkInstanceForPlatform {
          mkQemuCmd = loader: [
            "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-riscv64"
              "-machine" "spike"
              "-cpu" "rv64" "-m" "size=4096M"
              "-nographic"
              "-serial" "mon:stdio"
              "-kernel" loader
          ];
        };
      };
    };

  riscv32 =
    let
    in rec {
      default = spike.default;

      spike =
        let
          mk =
            { mcs ? false
            }:
            let
            in
              mkWorld {
                inherit kernelLoaderConfig;
                kernelConfig = kernelConfigCommon // {
                  KernelArch = mkString "riscv";
                  KernelSel4Arch = mkString "riscv32";
                  KernelPlatform = mkString "spike";
                  KernelIsMCS = fromBool mcs;
                };
                canSimulate = true;
                mkInstanceForPlatform = platUtils.qemu.mkMkInstanceForPlatform {
                  mkQemuCmd = loader: [
                    "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-riscv32"
                      "-machine" "spike"
                      "-cpu" "rv32" "-m" "size=2000M"
                      "-nographic"
                      "-serial" "mon:stdio"
                      "-bios" "${opensbi}/share/opensbi/ilp32/generic/firmware/fw_dynamic.bin"
                      "-kernel" loader
                  ];
                };
              };
        in rec {
          default = legacy;
          legacy = mk {};
          mcs = mk { mcs = true; };
        };
    };

  x86 = is64bit:
    let
    in rec {
      default = pc99;

      pc99 = lib.fix (self: mkWorld {
        inherit kernelLoaderConfig;
        platformRequiresLoader = false;
        kernelConfig = kernelConfigCommon // {
          KernelArch = mkString "x86";
          KernelSel4Arch = mkString (if is64bit then "x86_64" else "ia32");
          KernelPlatform = mkString "pc99";

          # for the sake of simulation (see seL4_tools/cmake-tool/helpers/application_settings.cmake)
          KernelFSGSBase = mkString "msr";
          # KernelFSGSBase = mkString "inst";
          KernelSupportPCID = off;
          KernelIOMMU = off;
          KernelFPU = mkString "FXSAVE";
        };
        canSimulate = true;
        mkInstanceForPlatform = platUtils.qemu.mkMkInstanceForPlatform {
          platformRequiresLoader = false;
          mkQemuCmd =
            let
              enable = opt: "+${opt}";
              disable = opt: "-${opt}";
              opts = lib.concatStringsSep "," [
                (disable "vme")
                (enable "pdpe1gb")
                (disable "xsave")
                (disable "xsaveopt")
                (disable "xsavec")
                (disable "fsgsbase")
                # (enable "fsgsbase")
                (disable "invpcid")
                (enable "syscall")
                (enable "lm")
              ];
            in task: [
              "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-${if is64bit then "x86_64" else "i386"}"
                "-cpu" "Nehalem,${opts},enforce"
                "-m" "size=512M"
                "-nographic"
                "-serial" "mon:stdio"
                "-kernel" self.kernelBinary32Bit
                "-initrd" task
            ];
        };
      });
    };

  x86_64 = x86 true;
  ia32 = x86 false;

}
