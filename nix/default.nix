let
  nixpkgsFn = import (builtins.getFlake "nixpkgs/227de2b3bbec142f912c09d5e8a1b4e778aa54fb").outPath;

  mkPkgs = crossSystem: nixpkgsFn {
    inherit crossSystem;
    overlays = [
      (self: super: {
        pythonPackagesExtensions = super.pythonPackagesExtensions ++ [
          (self.callPackage ./python-overrides.nix {})
        ];
      })
    ];
  };

  pkgs = {
    dev = mkPkgs null;
    aarch64 = mkPkgs { config = "aarch64-none-elf"; };
    riscv64 = mkPkgs { config = "riscv64-none-elf"; };
    # Use this instead of x86_64-none-elf to avoid cache miss at the expense of uniformity
    x86_64 = mkPkgs { config = "x86_64-unknown-linux-gnu"; };
  };

  inherit (pkgs.dev) lib;

  configHelpers = pkgs.dev.callPackage ./config-helpers.nix {};

  mkWorld =
    { crossPkgs, kernelConfig, loaderConfig
    , rustSeL4Target, rustBareMetalTarget
    , qemuCmd
    , ...
    } @ args:
    let
    in args // rec {
      kernel = crossPkgs.callPackage ./kernel.nix {} {
        config = kernelConfig;
      };
      shell = crossPkgs.callPackage ./shell.nix {} {
        inherit kernel loaderConfig;
        inherit rustSeL4Target rustBareMetalTarget;
        inherit qemuCmd;
      };
    };

  worlds =
    with configHelpers;
    let
      loaderConfig = {};
      noQemuCmd = [ "false" ];
    in {
      aarch64 =
        let
          crossPkgs = pkgs.aarch64;
          rustBareMetalTarget = "aarch64-unknown-none";
          rustSeL4Target = "aarch64-unknown-sel4";
        in rec {
          default = qemu-arm-virt.el2;
          qemu-arm-virt =
            let
              mk = hypervisor: mkWorld {
                inherit crossPkgs loaderConfig;
                inherit rustSeL4Target rustBareMetalTarget;
                kernelConfig = {
                  ARM_CPU = mkString "cortex-a57";
                  KernelArch = mkString "arm";
                  KernelSel4Arch = mkString "aarch64";
                  KernelPlatform = mkString "qemu-arm-virt";
                  KernelMaxNumNodes = mkString "2";
                  KernelVerificationBuild = off;
                } // lib.optionalAttrs hypervisor {
                  KernelArmHypervisorSupport = on;
                };
                qemuCmd = [
                  # NOTE
                  # virtualization=on even when hypervisor to test loader dropping exception level
                  "qemu-system-aarch64"
                    "-machine" "virt,virtualization=on"
                    "-cpu" "cortex-a57" "-smp" "2" "-m" "1024"
                    "-nographic"
                    "-serial" "mon:stdio"
                ];
              };
            in rec {
              default = el2;
              el1 = mk false;
              el2 = mk true;
            };
          bcm2711 = mkWorld {
            inherit crossPkgs loaderConfig;
            inherit rustSeL4Target rustBareMetalTarget;
            kernelConfig = {
              ARM_CPU = mkString "cortex-a57";
              KernelArch = mkString "arm";
              KernelSel4Arch = mkString "aarch64";
              KernelPlatform = mkString "bcm2711";
              KernelArmHypervisorSupport = on;
              # KernelMaxNumNodes = mkString "2"; # TODO
              KernelVerificationBuild = off;
            };
            qemuCmd = noQemuCmd;
          };
        };
      riscv64 =
        let
          crossPkgs = pkgs.riscv64;
          rustBareMetalTarget = "riscv64imac-unknown-none-elf";
          rustSeL4Target = rustBareMetalTarget; # TODO
        in rec {
          default = spike;
          spike = mkWorld {
            inherit crossPkgs loaderConfig;
            inherit rustSeL4Target rustBareMetalTarget;
            kernelConfig = {
              KernelArch = mkString "riscv";
              KernelSel4Arch = mkString "riscv64";
              KernelPlatform = mkString "spike";
              KernelVerificationBuild = off;
            };
            qemuCmd = noQemuCmd; # TODO
          };
        };
      x86_64 =
        let
          crossPkgs = pkgs.x86_64;
          rustBareMetalTarget = "x86_64-unknown-none";
          rustSeL4Target = rustBareMetalTarget; # TODO
        in rec {
          default = pc99;
          pc99 = lib.fix (self: mkWorld {
            inherit crossPkgs loaderConfig;
            inherit rustSeL4Target rustBareMetalTarget;
            kernelConfig = {
              KernelArch = mkString "x86";
              KernelSel4Arch = mkString "x86_64";
              KernelPlatform = mkString "pc99";
              KernelVerificationBuild = off;

              # for the sake of simulation (see seL4_tools/cmake-tool/helpers/application_settings.cmake)
              KernelFSGSBase = mkString "msr";
              KernelSupportPCID = off;
              KernelIOMMU = off;
              KernelFPU = mkString "FXSAVE";
            };
            qemuCmd =
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
              in [
                "qemu-system-x86_64"
                  "-cpu" "Nehalem,${opts},enforce"
                  "-m" "size=512M"
                  "-nographic"
                  "-serial" "mon:stdio"
                  "-kernel" "${self.kernel32Bit}"
              ];
            kernel32Bit = crossPkgs.runCommandCC "kernel32.elf" {} ''
              $OBJCOPY -O elf32-i386 ${self.kernel}/bin/kernel.elf $out
            '';
          });
        };
    };

in {
  inherit pkgs worlds;
}
