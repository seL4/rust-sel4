#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

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
    # KernelRootCNodeSizeBits = mkString "14"; # For backtrace test with embedded debug info
    KernelRootCNodeSizeBits = mkString "20"; # For ring test
  };

  kernelLoaderConfig = {};

  microkitConfig = {};

in rec {
  aarch64 =
    let
    in rec {
      default = qemu-arm-virt.default;
      qemu-arm-virt =
        let
          mk =
            { verificationBuild ? null
            , debugBuild ? null
            , printing ? null
            , benchmarks ? null

            , smp ? false
            , mcs ? false
            , el2 ? true
            , hypervisor ? false
            , cpu ? "cortex-a57"
            , mkLoaderQEMUArgs ? loader: [ "-kernel" loader ]

            , isMicrokit ? false
            , microkitBoard ? null
            , microkitConfig ? if debugBuild == null || debugBuild then "debug" else "release"

            , extraQEMUArgs ? []
            }:
            let
              numCores = if smp then "2" else "1";
            in
              mkWorld {
                inherit kernelLoaderConfig;
                inherit isMicrokit;
                microkitConfig = {
                  board = microkitBoard;
                  config = microkitConfig;
                };
                kernelConfig = kernelConfigCommon // {
                  ARM_CPU = mkString cpu;
                  KernelArch = mkString "arm";
                  KernelSel4Arch = mkString "aarch64";
                  KernelPlatform = mkString "qemu-arm-virt";
                  KernelMaxNumNodes = mkString numCores;
                  KernelIsMCS = fromBool mcs;
                } // lib.optionalAttrs hypervisor {
                  KernelArmHypervisorSupport = on;
                } // lib.optionalAttrs (verificationBuild != null) {
                  KernelVerificationBuild = fromBool verificationBuild;
                } // lib.optionalAttrs (debugBuild != null) {
                  KernelDebugBuild = fromBool debugBuild;
                } // lib.optionalAttrs (printing != null) {
                  KernelPrinting = fromBool printing;
                } // lib.optionalAttrs (benchmarks != null) {
                  KernelBenchmarks = mkString benchmarks;
                };
                  # benchmarking config
                  # KernelDebugBuild = off;
                  # KernelBenchmarks = mkString "track_utilisation";
                  # KernelArmExportPMUUser = on;
                  # KernelSignalFastpath = on;
                canSimulate = true;
                mkPlatformSystemExtension = platUtils.qemu.mkMkPlatformSystemExtension {
                  mkQEMUCmd = loader: [
                    # NOTE
                    # virtualization=on even when hypervisor to test loader dropping exception level
                    "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-aarch64"
                      "-machine" "virt${lib.optionalString el2 ",virtualization=on"}"
                      "-cpu" cpu "-smp" numCores "-m" "size=1024"
                      "-nographic"
                      "-serial" "mon:stdio"
                  ] ++ mkLoaderQEMUArgs loader ++ extraQEMUArgs;
                };
              };
        in rec {
          default = el2;
          el1 = mk { smp = true; };
          el2 = mk { smp = true; hypervisor = true; };
          el2MCS = mk { smp = true; hypervisor = true; mcs = true; };
          microkit = mk {
            el2 = false;
            mcs = true;
            isMicrokit = true;
            microkitBoard = "qemu_virt_aarch64";
            cpu = "cortex-a53";
            mkLoaderQEMUArgs = loader: [ "-device" "loader,file=${loader},addr=0x70000000,cpu-num=0" ];
            extraQEMUArgs = [ "-m" "size=2G" ];
          };

          forBuildTests = {
            noPrinting = mk {
              debugBuild = true;
              printing = false;
            };
            noDebug = mk {
              debugBuild = false;
              printing = true;
            };
            verification = mk {
              verificationBuild = true;
            };
            bench = mk {
              verificationBuild = true;
              benchmarks = "track_utilisation";
            };
            debugBench = mk {
              debugBuild = true;
              benchmarks = "track_utilisation";
            };
            mcs = mk {
              mcs = true;
            };
          };
        };

      bcm2711 = mkWorld {
        inherit kernelLoaderConfig;
        kernelConfig = kernelConfigCommon // {
          RPI4_MEMORY = mkString "2048"; # match QEMU's model
          KernelArch = mkString "arm";
          KernelSel4Arch = mkString "aarch64";
          KernelPlatform = mkString "bcm2711";
          KernelArmHypervisorSupport = on; # seems incompatible with QEMU's model
          KernelMaxNumNodes = mkString "4"; # currently not working with QEMU
        };
        mkPlatformSystemExtension = platUtils.rpi4.mkPlatformSystemExtension;
      };

      zcu102 =
        let
          mk = { isMicrokit ? false }:
            mkWorld ({
              inherit isMicrokit;
            } // (if isMicrokit then {
              microkitConfig = {
                board = "zcu102";
                config = "debug";
              };
            } else {
              kernelConfig = kernelConfigCommon // {
                KernelSel4Arch = mkString "aarch64";
                KernelPlatform = mkString "zynqmp";
                KernelARMPlatform = mkString "zcu102";
              };
            }) // {
              canSimulate = true;
              mkPlatformSystemExtension = platUtils.qemu.mkMkPlatformSystemExtension {
                mkQEMUCmd = loader: [
                  "${pkgsBuildBuild.this.qemuForSeL4Xilinx}/bin/qemu-system-aarch64"
                  # "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-aarch64"
                    "-machine" "xlnx-zcu102"
                    # "-machine" "arm-generic-fdt"
                    # "-hw-dtb" "${pkgsBuildBuild.this.qemuForSeL4Xilinx.devicetrees}/SINGLE_ARCH//zcu102-arm.dtb"
                    "-m" "size=4G"
                    "-nographic"
                    "-serial" "mon:stdio"
                ] ++ (if isMicrokit then [
                    "-device" "loader,file=${loader},addr=0x40000000,cpu-num=0"
                    "-device" "loader,addr=0xfd1a0104,data=0x0000000e,data-len=4"
                ] else [
                    "-kernel" loader
                ]);
              };
            });
        in {
          default = mk {};
          microkit = mk { isMicrokit = true; };
        };
    };

  aarch32 =
    let
    in rec {
      default = qemu-arm-virt.default;
      qemu-arm-virt =
        let
          mk =
            { smp ? false
            , mcs ? false
            , bootInHyp ? true
            , cpu ? "cortex-a15"
            }:
            let
              numCores = if smp then "2" else "1";
            in
              mkWorld {
                inherit kernelLoaderConfig;
                isMicrokit = false;
                kernelConfig = kernelConfigCommon // {
                  ARM_CPU = mkString cpu;
                  KernelArch = mkString "arm";
                  KernelSel4Arch = mkString "aarch32";
                  KernelPlatform = mkString "qemu-arm-virt";
                  KernelMaxNumNodes = mkString numCores;
                  KernelIsMCS = fromBool mcs;
                };
                canSimulate = true;
                mkPlatformSystemExtension = platUtils.qemu.mkMkPlatformSystemExtension {
                  mkQEMUCmd = loader: [
                    "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-arm"
                      "-machine" "virt,highmem=off,secure=off,virtualization=${if bootInHyp then "on" else "off"}"
                      "-cpu" cpu "-smp" numCores "-m" "size=1024"
                      "-nographic"
                      "-serial" "mon:stdio"
                      "-kernel" loader
                  ];
                };
              };
        in rec {
          default = smp;
          legacy = mk {};
          smp = mk { smp = true; };
          mcs = mk { mcs = true; };
        };

      bcm2711 = mkWorld {
        inherit kernelLoaderConfig;
        kernelConfig = kernelConfigCommon // {
          RPI4_MEMORY = mkString "1024";
          KernelArch = mkString "arm";
          KernelSel4Arch = mkString "aarch32";
          KernelPlatform = mkString "bcm2711";
          KernelArmHypervisorSupport = off;
          KernelMaxNumNodes = mkString "4";
        };
        mkPlatformSystemExtension = platUtils.rpi4.mkPlatformSystemExtension;
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
                inherit kernelLoaderConfig microkitConfig;
                kernelConfig = kernelConfigCommon // {
                  QEMU_MEMORY = mkString qemuMemory;
                  KernelArch = mkString "riscv";
                  KernelSel4Arch = mkString "riscv64";
                  KernelPlatform = mkString "qemu-riscv-virt";
                  KernelMaxNumNodes = mkString numCores;
                  KernelIsMCS = fromBool mcs;
                };
                canSimulate = true;
                mkPlatformSystemExtension = platUtils.qemu.mkMkPlatformSystemExtension {
                  mkQEMUCmd = loader: [
                    "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-riscv64"
                      "-machine" "virt"
                      "-cpu" "rv64" "-smp" numCores "-m" "size=${qemuMemory}"
                      "-nographic"
                      "-serial" "mon:stdio"
                      "-kernel" loader
                      # "-d" "unimp,guest_errors"
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
        mkPlatformSystemExtension = platUtils.qemu.mkMkPlatformSystemExtension {
          mkQEMUCmd = loader: [
            "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-riscv64"
              "-machine" "spike"
              "-cpu" "rv64" "-m" "size=4096M"
              "-nographic"
              "-serial" "mon:stdio"
              "-kernel" loader
            # TODO
            # "${pkgsBuildBuild.spike}/bin/spike"
            #   "-m4096"
            #   "--kernel=${loader}"
            #   "${opensbi}/share/opensbi/lp64/generic/firmware/fw_dynamic.elf"
          ];
        };
      };

      hifive = mkWorld {
        inherit kernelLoaderConfig;
        kernelConfig = kernelConfigCommon // {
          KernelArch = mkString "riscv";
          KernelSel4Arch = mkString "riscv64";
          KernelPlatform = mkString "hifive";
        };
        canSimulate = true;
        mkPlatformSystemExtension = platUtils.qemu.mkMkPlatformSystemExtension {
          mkQEMUCmd = loader: [
            "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-riscv64"
              "-machine" "sifive_u"
              "-m" "size=8192M"
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
                mkPlatformSystemExtension = platUtils.qemu.mkMkPlatformSystemExtension {
                  mkQEMUCmd = loader: [
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
        mkPlatformSystemExtension = platUtils.qemu.mkMkPlatformSystemExtension {
          mkQEMUCmd =
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
