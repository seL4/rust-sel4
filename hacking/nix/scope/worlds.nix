{ lib, pkgsBuildBuild
, configHelpers
, mkWorld
}:

with configHelpers;

let
  loaderConfig = {};

  # smp = false;
  smp = true;

  kernelConfigCommon = {
    KernelIsMCS = on;
    KernelVerificationBuild = off;
  };

in rec {
  aarch64 =
    let
    in rec {
      default = qemu-arm-virt.el2;
      qemu-arm-virt =
        let
          numCores = if smp then "2" else "1";
          mk = hypervisor: mkWorld {
            inherit loaderConfig;
            kernelConfig = kernelConfigCommon // {
              ARM_CPU = mkString "cortex-a57";
              KernelArch = mkString "arm";
              KernelSel4Arch = mkString "aarch64";
              KernelPlatform = mkString "qemu-arm-virt";
              KernelMaxNumNodes = mkString numCores;
            } // lib.optionalAttrs hypervisor {
              KernelArmHypervisorSupport = on;
            };
            mkQemuCmd = loader: [
              # NOTE
              # virtualization=on even when hypervisor to test loader dropping exception level
              "${pkgsBuildBuild.qemu}/bin/qemu-system-aarch64"
                "-machine" "virt,virtualization=on"
                "-cpu" "cortex-a57" "-smp" numCores "-m" "1024"
                "-nographic"
                "-serial" "mon:stdio"
                "-kernel" loader
            ];
          };
        in rec {
          default = el2;
          el1 = mk false;
          el2 = mk true;
        };

      bcm2711 = mkWorld {
        inherit loaderConfig;
        kernelConfig = kernelConfigCommon // {
          ARM_CPU = mkString "cortex-a57";
          KernelArch = mkString "arm";
          KernelSel4Arch = mkString "aarch64";
          KernelPlatform = mkString "bcm2711";
          KernelArmHypervisorSupport = on;
          # KernelMaxNumNodes = mkString "2"; # TODO
        };
      };
    };

  riscv64 =
    let
    in rec {
      default = spike;

      spike = mkWorld {
        inherit loaderConfig;
        kernelConfig = kernelConfigCommon // {
          KernelArch = mkString "riscv";
          KernelSel4Arch = mkString "riscv64";
          KernelPlatform = mkString "spike";
        };
      };
    };

  x86_64 =
    let
    in rec {
      default = pc99;

      pc99 = lib.fix (self: mkWorld {
        inherit loaderConfig;
        kernelConfig = kernelConfigCommon // {
          KernelArch = mkString "x86";
          KernelSel4Arch = mkString "x86_64";
          KernelPlatform = mkString "pc99";

          # for the sake of simulation (see seL4_tools/cmake-tool/helpers/application_settings.cmake)
          KernelFSGSBase = mkString "msr";
          KernelSupportPCID = off;
          KernelIOMMU = off;
          KernelFPU = mkString "FXSAVE";
        };
        qemuCmdRequiresLoader = false;
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
              (disable "invpcid")
              (enable "syscall")
              (enable "lm")
            ];
          in task: [
            "${pkgsBuildBuild.qemu}/bin/qemu-system-x86_64"
              "-cpu" "Nehalem,${opts},enforce"
              "-m" "size=512M"
              "-nographic"
              "-serial" "mon:stdio"
              "-kernel" self.kernel32Bit
              "-initrd" task
          ];
      });
    };
}
